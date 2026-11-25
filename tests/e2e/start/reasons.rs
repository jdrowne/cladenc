use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use crate::shared::STOPPED;

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
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

static ON_A_MACHINE_WHERE_DOCKER_IS_NOT_FOUND: LazyLock<Output> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit 0")
        .env("PATH", directory.path())
        .output()
        .unwrap()
});

fn said_last(ended: &Output) -> String {
    let said = str::from_utf8(&ended.stderr).unwrap();
    said.lines().last().unwrap_or_default().to_owned()
}

#[test]
fn a_run_a_container_stopped_says_which_attempt_and_where_what_docker_said_is_kept() {
    let (directory, ended) = &*STOPPED;
    assert_eq!(
        said_last(ended),
        format!(
            "cladenc: the directed evolution stopped before its end: the attempt of harness 0 \
             on task benchmark/t1 could not be carried out: Docker did not start its container, \
             and {} holds what Docker said",
            directory
                .path()
                .join("outputs/running/attempt-0-benchmark+t1/stderr")
                .display()
        )
    );
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_says_that_the_first_environment_is_not_built() {
    assert_eq!(
        said_last(&ON_A_MACHINE_WHERE_DOCKER_IS_NOT_FOUND),
        "cladenc: the environment of task benchmark/t1 was not built: its build could not be \
         carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_exits_with_2() {
    assert_eq!(
        ON_A_MACHINE_WHERE_DOCKER_IS_NOT_FOUND.status.code(),
        Some(2)
    );
}
