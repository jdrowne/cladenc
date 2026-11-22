use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use tempfile::TempDir;

fn start(directory: &Path) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn not_built() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN exit 3\n", test_inputs::IMAGE),
    )
    .unwrap();
    let output = start.output().unwrap();
    (directory, output)
}

fn arguments_of_a_docker_that_records_them() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path())
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    (directory, recorded.lines().map(str::to_owned).collect())
}

fn builds(arguments: &[String]) -> Vec<&[String]> {
    arguments
        .split(|argument| argument == "build")
        .skip(1)
        .map(|build| build.split(|argument| argument == "run").next().unwrap())
        .collect()
}

fn is_named_by_cladenc(image: &str) -> bool {
    image
        .strip_prefix("cladenc-environments-")
        .and_then(|rest| rest.strip_suffix(":0"))
        .is_some_and(|digits| {
            digits.len() == 16
                && digits
                    .chars()
                    .all(|digit| matches!(digit, '0'..='9' | 'a'..='f'))
        })
}

#[test]
fn a_task_whose_environment_is_not_built_exits_with_2() {
    assert_eq!(not_built().1.status.code(), Some(2));
}

#[test]
fn a_task_whose_environment_is_not_built_is_reported_last_on_standard_error_with_its_task() {
    assert!(
        String::from_utf8(not_built().1.stderr)
            .unwrap()
            .lines()
            .last()
            .unwrap_or_default()
            .starts_with(
                "cladenc: the environment of task benchmark/t2 was not built: its build exited \
                 with "
            )
    );
}

#[test]
fn a_task_whose_environment_is_not_built_prints_nothing_on_standard_output() {
    assert_eq!(not_built().1.stdout, b"");
}

#[test]
fn a_task_whose_environment_is_not_built_leaves_no_journal() {
    let (directory, _output) = not_built();
    assert!(!directory.path().join("outputs/journal.jsonl").exists());
}

#[test]
fn docker_is_asked_to_build_each_environment_from_the_copy_of_the_task_labelled_with_the_task() {
    let (directory, arguments) = arguments_of_a_docker_that_records_them();
    let task = directory.path().join("outputs/benchmarks/benchmark/t2");
    assert_eq!(
        [&builds(&arguments)[1][..1], &builds(&arguments)[1][2..]].concat(),
        [
            "--tag".to_owned(),
            "--label".to_owned(),
            "cladenc.task=benchmark/t2".to_owned(),
            task.join("environment").display().to_string(),
        ]
    );
}

#[test]
fn the_environment_of_each_task_is_tagged_with_the_place_of_the_task_among_the_tasks() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert_eq!(
        builds(&arguments)
            .iter()
            .map(|build| build[1].rsplit(':').next().unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["0", "1", "2"]
    );
}

#[test]
fn the_environments_of_a_directed_evolution_share_a_repository_named_by_cladenc() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert!(is_named_by_cladenc(&builds(&arguments)[0][1]));
}

#[test]
fn docker_is_not_asked_to_fetch_when_it_builds_an_environment() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert!(
        !builds(&arguments)
            .concat()
            .iter()
            .any(|argument| argument.starts_with("--pull"))
    );
}

fn environments_left_by(said: &str) -> Vec<String> {
    let repository = said
        .lines()
        .find_map(|line| line.split(" as the image ").nth(1))
        .and_then(|image| image.rsplit_once(':'))
        .map(|(repository, _tag)| repository.to_owned())
        .unwrap();
    let listed = Command::new("docker")
        .args(["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"])
        .arg(repository)
        .output()
        .unwrap();
    String::from_utf8(listed.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn interrupted_while_the_environment_of_t2_is_built() -> Output {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN sleep 300\n", test_inputs::IMAGE),
    )
    .unwrap();
    let cladenc = start
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let printed = directory
        .path()
        .join("outputs/environments/benchmark/t2/stdout");
    test_inputs::within_ten_seconds(|| printed.exists());
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGINT).unwrap();
    cladenc.wait_with_output().unwrap()
}

#[test]
fn cladenc_start_leaves_no_environment_it_built_on_the_machine_when_it_ends() {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path()).output().unwrap();
    assert_eq!(
        environments_left_by(&String::from_utf8(output.stderr).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn an_interrupt_while_an_environment_is_built_gives_exit_status_2() {
    assert_eq!(
        interrupted_while_the_environment_of_t2_is_built()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn an_interrupt_while_an_environment_is_built_says_that_resume_has_nothing_to_continue() {
    assert!(
        String::from_utf8(interrupted_while_the_environment_of_t2_is_built().stderr)
            .unwrap()
            .ends_with(
                "cladenc: the directed evolution was stopped before it was started, so `cladenc \
                 resume` has nothing to continue\n"
            )
    );
}
