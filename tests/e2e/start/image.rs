use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::symlink;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_EVERY_TASK: &str =
    "RUN for task in t1 t2 t3; do echo \"$task\"; done > /solved-by-the-image\n";
const FAILS: &str = "RUN exit 3\n";
const STEP_THAT_STAYS: &str = "sleep 307";
const LOOK: Duration = Duration::from_millis(100);
const LOOKS_IN_A_MINUTE: u32 = 600;

struct Killed {
    _directory: TempDir,
    image: Image,
    status: ExitStatus,
    the_step_was_running: bool,
}

struct Ran {
    directory: TempDir,
    image: String,
    the_image_stayed: bool,
    status: Option<i32>,
    output: String,
    said: String,
}

static SOLVED: LazyLock<Ran> = LazyLock::new(|| ran(SOLVES_EVERY_TASK));

static NOT_BUILT: LazyLock<Ran> = LazyLock::new(|| ran(FAILS));

fn start(directory: &Path, steps: &str) -> Command {
    start_within(directory, steps, "60")
}

fn start_within(directory: &Path, steps: &str, build_timeout: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        (
            "--procreation-dockerfile",
            test_inputs::dockerfile(directory, steps),
        ),
        (
            "--procreation-docker-image-build-timeout",
            build_timeout.into(),
        ),
        (
            "--harness-run-command",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES.into(),
        ),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn image_named_in(said: &str) -> Option<Image> {
    said.lines()
        .find_map(|line| line.strip_prefix("cladenc: building the image "))
        .and_then(|rest| rest.split(' ').next())
        .map(|name| Image {
            name: name.to_owned(),
        })
}

fn ran(steps: &str) -> Ran {
    let directory = test_inputs::temporary_directory();
    let start = start(directory.path(), steps);
    ended(directory, start)
}

static ON_A_MACHINE_THAT_DOES_NOT_HOLD_TIMEOUT: LazyLock<Ran> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), SOLVES_EVERY_TASK);
    start.env("PATH", programs_that_are_docker_alone(directory.path()));
    ended(directory, start)
});

fn programs_that_are_docker_alone(directory: &Path) -> PathBuf {
    let docker = env::split_paths(&env::var_os("PATH").unwrap())
        .map(|programs| programs.join("docker"))
        .find(|docker| docker.is_file())
        .unwrap();
    let programs = directory.join("programs");
    fs::create_dir(&programs).unwrap();
    symlink(docker, programs.join("docker")).unwrap();
    programs
}

fn ended(directory: TempDir, mut start: Command) -> Ran {
    let output = start.output().unwrap();
    let said = String::from_utf8(output.stderr).unwrap();
    let image = image_named_in(&said);
    Ran {
        directory,
        image: image
            .as_ref()
            .map_or_else(String::new, |image| image.name.clone()),
        the_image_stayed: image
            .as_ref()
            .is_some_and(|image| docker_holds(&image.name)),
        status: output.status.code(),
        output: String::from_utf8(output.stdout).unwrap(),
        said,
    }
}

fn is_named_by_cladenc(image: &str) -> bool {
    image.strip_prefix("cladenc-").is_some_and(|digits| {
        digits.len() == 16
            && digits
                .chars()
                .all(|digit| matches!(digit, '0'..='9' | 'a'..='f'))
    })
}

fn docker_holds(image: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", image])
        .output()
        .unwrap()
        .status
        .success()
}

fn kept_of_the_image(ran: &Ran) -> Vec<String> {
    let mut kept: Vec<String> = fs::read_dir(ran.directory.path().join("outputs/image"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    kept.sort();
    kept
}

fn the_step_is_running() -> bool {
    fs::read_dir("/proc").unwrap().flatten().any(|process| {
        fs::read(process.path().join("cmdline")).is_ok_and(|command| {
            String::from_utf8_lossy(&command)
                .replace('\0', " ")
                .contains(STEP_THAT_STAYS)
        })
    })
}

static KILLED_OUTRIGHT_WHILE_IT_BUILDS: LazyLock<Killed> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let steps = format!("RUN {STEP_THAT_STAYS}\n");
    let mut cladenc = start_within(directory.path(), &steps, "12")
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let image = BufReader::new(cladenc.stderr.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .find_map(|line| image_named_in(&line))
        .unwrap();
    let the_step_was_running = test_inputs::within_ten_seconds(the_step_is_running);
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGKILL).unwrap();
    let status = cladenc.wait().unwrap();
    Killed {
        _directory: directory,
        image,
        status,
        the_step_was_running,
    }
});

#[test]
fn a_cladenc_that_is_killed_outright_while_it_builds_the_image_is_ended_by_the_signal() {
    assert_eq!(
        KILLED_OUTRIGHT_WHILE_IT_BUILDS.status.signal(),
        Some(Signal::SIGKILL as i32)
    );
}

#[test]
fn the_build_of_the_image_ends_itself_at_its_time_limit_when_cladenc_was_killed_outright() {
    let killed = &*KILLED_OUTRIGHT_WHILE_IT_BUILDS;
    let the_step_ended = (0..LOOKS_IN_A_MINUTE).any(|_| {
        thread::sleep(LOOK);
        !the_step_is_running()
    });
    assert_eq!(
        [
            killed.the_step_was_running,
            the_step_ended,
            docker_holds(&killed.image.name)
        ],
        [true, true, false]
    );
}

#[test]
fn a_machine_that_does_not_hold_timeout_builds_no_image_and_exits_with_2() {
    assert_eq!(ON_A_MACHINE_THAT_DOES_NOT_HOLD_TIMEOUT.status, Some(2));
}

#[test]
fn a_machine_that_does_not_hold_timeout_is_told_that_the_build_could_not_be_carried_out() {
    assert_eq!(
        ON_A_MACHINE_THAT_DOES_NOT_HOLD_TIMEOUT
            .said
            .lines()
            .last()
            .unwrap_or_default(),
        "cladenc: the image of the directed evolution was not built: its build could not be \
         carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_directed_evolution_given_a_dockerfile_exits_with_0() {
    assert_eq!(SOLVED.status, Some(0));
}

#[test]
fn a_directed_evolution_given_a_dockerfile_prints_the_harness_made_in_the_image_it_built() {
    assert_eq!(
        SOLVED.output,
        format!(
            "{}\n",
            SOLVED
                .directory
                .path()
                .join("outputs/harnesses/1")
                .display()
        )
    );
}

#[test]
fn the_image_cladenc_builds_is_named_cladenc_and_sixteen_hexadecimal_digits() {
    assert!(is_named_by_cladenc(&SOLVED.image));
}

#[test]
fn the_image_stays_on_the_machine_when_the_directed_evolution_ends() {
    assert!(SOLVED.the_image_stayed);
}

#[test]
fn the_outputs_directory_keeps_the_dockerfile_and_what_docker_printed() {
    assert_eq!(
        kept_of_the_image(&SOLVED),
        ["Dockerfile", "context", "stderr", "stdout"]
    );
}

#[test]
fn a_dockerfile_that_is_not_built_exits_with_2() {
    assert_eq!(NOT_BUILT.status, Some(2));
}

#[test]
fn a_dockerfile_that_is_not_built_is_reported_last_on_standard_error() {
    assert!(NOT_BUILT.said.lines().last().unwrap().starts_with(
        "cladenc: the image of the directed evolution was not built: its build exited with "
    ));
}

#[test]
fn a_dockerfile_that_is_not_built_prints_nothing_on_standard_output() {
    assert_eq!(NOT_BUILT.output, "");
}

#[test]
fn a_dockerfile_that_is_not_built_leaves_no_journal() {
    assert!(
        !NOT_BUILT
            .directory
            .path()
            .join("outputs/journal.jsonl")
            .exists()
    );
}
