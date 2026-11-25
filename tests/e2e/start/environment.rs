use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::LazyLock;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

use crate::shared::UNCHANGED;

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

static NOT_BUILT: LazyLock<(TempDir, Output)> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN exit 3\n", test_inputs::IMAGE),
    )
    .unwrap();
    let output = start.output().unwrap();
    (directory, output)
});

#[test]
fn a_task_whose_environment_is_not_built_exits_with_2() {
    assert_eq!(NOT_BUILT.1.status.code(), Some(2));
}

#[test]
fn a_task_whose_environment_is_not_built_is_reported_last_on_standard_error_with_its_task() {
    assert!(
        str::from_utf8(&NOT_BUILT.1.stderr)
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
    assert_eq!(NOT_BUILT.1.stdout, b"");
}

#[test]
fn a_task_whose_environment_is_not_built_leaves_no_journal() {
    let (directory, _output) = &*NOT_BUILT;
    assert!(!directory.path().join("outputs/journal.jsonl").exists());
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

static INTERRUPTED_WHILE_THE_ENVIRONMENT_OF_T2_IS_BUILT: LazyLock<Output> = LazyLock::new(|| {
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
});

#[test]
fn cladenc_start_leaves_no_environment_it_built_on_the_machine_when_it_ends() {
    assert_eq!(
        environments_left_by(str::from_utf8(&UNCHANGED.1.stderr).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn an_interrupt_while_an_environment_is_built_gives_exit_status_2() {
    assert_eq!(
        INTERRUPTED_WHILE_THE_ENVIRONMENT_OF_T2_IS_BUILT
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn an_interrupt_while_an_environment_is_built_says_that_resume_has_nothing_to_continue() {
    assert!(
        str::from_utf8(&INTERRUPTED_WHILE_THE_ENVIRONMENT_OF_T2_IS_BUILT.stderr)
            .unwrap()
            .ends_with(
                "cladenc: the directed evolution was stopped before it was started, so `cladenc \
                 resume` has nothing to continue\n"
            )
    );
}
