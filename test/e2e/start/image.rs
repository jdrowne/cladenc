use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_T1: &str = "RUN echo t1 > /solved-by-the-image\n";
const FAILS: &str = "RUN exit 3\n";
const WAITS_FOURTEEN_SECONDS: &str = "RUN sleep 14\n";
const UNTIL_THE_STEP_WOULD_HAVE_ENDED: Duration = Duration::from_secs(20);

struct Killed {
    _directory: TempDir,
    image: Image,
    status: ExitStatus,
}

struct Ran {
    directory: TempDir,
    image: Option<Image>,
    status: Option<i32>,
    output: String,
    said: String,
}

fn start(directory: &Path, steps: &str) -> Command {
    start_within(directory, steps, "60")
}

fn start_within(directory: &Path, steps: &str, build_timeout: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--dockerfile", test_inputs::dockerfile(directory, steps)),
        ("--docker-build-timeout", build_timeout.into()),
        (
            "--harness-run-command",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES.into(),
        ),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
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
    ran_on_a_machine_whose_programs_are_in(steps, &env::var("PATH").unwrap())
}

fn ran_on_a_machine_that_does_not_hold_timeout() -> Ran {
    let only_docker = test_inputs::sample("e2e", "fake");
    ran_on_a_machine_whose_programs_are_in(SOLVES_T1, &only_docker.display().to_string())
}

fn ran_on_a_machine_whose_programs_are_in(steps: &str, path: &str) -> Ran {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path(), steps)
        .env("PATH", path)
        .env(
            "CLADENC_TEST_ARGUMENTS_OF_DOCKER",
            directory.path().join("arguments-of-docker"),
        )
        .output()
        .unwrap();
    let said = String::from_utf8(output.stderr).unwrap();
    Ran {
        directory,
        image: image_named_in(&said),
        status: output.status.code(),
        output: String::from_utf8(output.stdout).unwrap(),
        said,
    }
}

fn name_of_the_image(ran: &Ran) -> &str {
    ran.image.as_ref().map_or("", |image| image.name.as_str())
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
    let mut kept: Vec<String> = fs::read_dir(ran.directory.path().join("run/image"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    kept.sort();
    kept
}

fn arguments_of_a_docker_that_records_them() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), SOLVES_T1)
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    (directory, recorded.lines().map(str::to_owned).collect())
}

fn arguments_of_a_timeout_that_records_them() -> Vec<String> {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-timeout");
    let path = format!(
        "{}:{}:{}",
        test_inputs::sample("e2e", "timed").display(),
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), SOLVES_T1)
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_TIMEOUT", &arguments)
        .env(
            "CLADENC_TEST_ARGUMENTS_OF_DOCKER",
            directory.path().join("arguments-of-docker"),
        )
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    recorded.lines().map(str::to_owned).collect()
}

fn killed_outright_while_it_builds() -> Killed {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start_within(directory.path(), WAITS_FOURTEEN_SECONDS, "3")
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let image = BufReader::new(cladenc.stderr.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .find_map(|line| image_named_in(&line))
        .unwrap();
    let printed = directory.path().join("run/image");
    test_inputs::within_ten_seconds(|| {
        test_inputs::line_is_written(&printed.join("stderr"))
            || test_inputs::line_is_written(&printed.join("stdout"))
    });
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGKILL).unwrap();
    let status = cladenc.wait().unwrap();
    Killed {
        _directory: directory,
        image,
        status,
    }
}

#[test]
fn a_cladenc_that_is_killed_outright_while_it_builds_the_image_is_ended_by_the_signal() {
    assert_eq!(
        killed_outright_while_it_builds().status.signal(),
        Some(Signal::SIGKILL as i32)
    );
}

#[test]
fn the_build_of_the_image_ends_itself_at_its_time_limit_when_cladenc_was_killed_outright() {
    let killed = killed_outright_while_it_builds();
    thread::sleep(UNTIL_THE_STEP_WOULD_HAVE_ENDED);
    assert!(!docker_holds(&killed.image.name));
}

#[test]
fn the_build_of_the_image_is_started_under_a_timeout_five_seconds_longer_than_its_time_limit() {
    assert_eq!(
        arguments_of_a_timeout_that_records_them()[..5],
        ["-s", "KILL", "65", "docker", "build"]
    );
}

#[test]
fn a_machine_that_does_not_hold_timeout_builds_no_image_and_exits_with_2() {
    assert_eq!(
        ran_on_a_machine_that_does_not_hold_timeout().status,
        Some(2)
    );
}

#[test]
fn a_machine_that_does_not_hold_timeout_is_told_that_the_build_could_not_be_carried_out() {
    assert_eq!(
        ran_on_a_machine_that_does_not_hold_timeout()
            .said
            .lines()
            .last()
            .unwrap_or_default(),
        "cladenc: the image of the search was not built: its build could not be carried out: No \
         such file or directory (os error 2)"
    );
}

#[test]
fn a_search_given_a_dockerfile_exits_with_0() {
    assert_eq!(ran(SOLVES_T1).status, Some(0));
}

#[test]
fn a_search_given_a_dockerfile_prints_the_harness_made_in_the_image_it_built() {
    let ran = ran(SOLVES_T1);
    assert_eq!(
        ran.output,
        format!(
            "{}\n",
            ran.directory.path().join("run/harnesses/1").display()
        )
    );
}

#[test]
fn the_image_cladenc_builds_is_named_cladenc_and_sixteen_hexadecimal_digits() {
    assert!(is_named_by_cladenc(name_of_the_image(&ran(SOLVES_T1))));
}

#[test]
fn the_image_stays_on_the_machine_when_the_search_ends() {
    assert!(docker_holds(name_of_the_image(&ran(SOLVES_T1))));
}

#[test]
fn the_run_directory_keeps_the_dockerfile_and_what_docker_printed() {
    assert_eq!(
        kept_of_the_image(&ran(SOLVES_T1)),
        ["Dockerfile", "context", "stderr", "stdout"]
    );
}

#[test]
fn a_dockerfile_that_is_not_built_exits_with_2() {
    assert_eq!(ran(FAILS).status, Some(2));
}

#[test]
fn a_dockerfile_that_is_not_built_is_reported_last_on_standard_error() {
    assert!(
        ran(FAILS)
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with("cladenc: the image of the search was not built: its build exited with ")
    );
}

#[test]
fn a_dockerfile_that_is_not_built_prints_nothing_on_standard_output() {
    assert_eq!(ran(FAILS).output, "");
}

#[test]
fn a_dockerfile_that_is_not_built_leaves_no_journal() {
    assert!(
        !ran(FAILS)
            .directory
            .path()
            .join("run/journal.jsonl")
            .exists()
    );
}

#[test]
fn docker_is_asked_to_build_from_the_copy_of_the_dockerfile_with_an_empty_directory_of_files() {
    let (directory, arguments) = arguments_of_a_docker_that_records_them();
    let kept = directory.path().join("run/image");
    assert_eq!(
        [&arguments[..2], &arguments[3..6]].concat(),
        [
            "build".to_owned(),
            "--tag".to_owned(),
            "--file".to_owned(),
            kept.join("Dockerfile").display().to_string(),
            kept.join("context").display().to_string(),
        ]
    );
}

#[test]
fn every_container_is_started_from_the_image_docker_was_asked_to_build() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    let started_from = format!("\n{}\nsh\n-c\n", arguments[2]);
    assert!(arguments.join("\n").contains(&started_from));
}

#[test]
fn docker_is_asked_to_build_once() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert_eq!(
        arguments
            .iter()
            .filter(|argument| *argument == "build")
            .count(),
        1
    );
}

#[test]
fn docker_is_not_asked_to_fetch_when_it_builds_the_image() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    let build = arguments
        .split(|argument| argument == "run")
        .next()
        .unwrap_or_default();
    assert!(!build.iter().any(|argument| argument.starts_with("--pull")));
}

#[test]
fn docker_is_told_never_to_fetch_the_image_it_built_at_any_container() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    let recorded = arguments.join("\n");
    let containers: Vec<&str> = recorded.split("\nrun\n").skip(1).collect();
    let told_never = containers
        .iter()
        .filter(|container| container.starts_with("--rm\n--pull\nnever\n"))
        .count();
    assert_eq!((told_never > 0, told_never), (true, containers.len()));
}
