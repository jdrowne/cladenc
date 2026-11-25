use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

const NEVER_READY: &str = "[environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
const READY: &str = "[environment.healthcheck]\ncommand = \"true\"\n";

fn start(directory: &Path, task_toml_of_t2: &str) -> Command {
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
        ("--max-task-health-check-retries", "2".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    fs::write(directory.join("benchmark/t2/task.toml"), task_toml_of_t2).unwrap();
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn with_a_task_that_is_never_ready() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path(), NEVER_READY).output().unwrap();
    (directory, output)
}

static RESUMED_ONCE_THE_TASK_IS_READY: LazyLock<Output> = LazyLock::new(|| {
    let (directory, _) = with_a_task_that_is_never_ready();
    let outputs = directory.path().join("outputs");
    fs::write(outputs.join("benchmarks/benchmark/t2/task.toml"), READY).unwrap();
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--outputs-dir={}", outputs.display()))
        .output()
        .unwrap()
});

static WITH_A_TASK_THAT_IS_NEVER_READY: LazyLock<(TempDir, Output)> =
    LazyLock::new(with_a_task_that_is_never_ready);

fn said(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn a_task_that_is_never_ready_stops_cladenc_start_with_exit_status_1() {
    assert_eq!(WITH_A_TASK_THAT_IS_NEVER_READY.1.status.code(), Some(1));
}

#[test]
fn a_task_that_is_never_ready_is_reported_last_with_all_that_says_what_happened() {
    let (directory, output) = &*WITH_A_TASK_THAT_IS_NEVER_READY;
    assert_eq!(
        said(output).lines().last().unwrap_or_default(),
        format!(
            "cladenc: the directed evolution stopped before its end: the attempt of harness 0 \
             on task benchmark/t2 was not carried out: the health check of the task did not \
             pass in 3 tries, so the harness was not started, and \
             {}/running/attempt-0-benchmark+t2/health-check holds what the health check printed \
             the last time it ran",
            directory.path().join("outputs").display()
        )
    );
}

#[test]
fn cladenc_start_says_each_retry_of_the_health_check_of_a_task() {
    let said = said(&WITH_A_TASK_THAT_IS_NEVER_READY.1);
    let retries = [
        "cladenc: the attempt of harness 0 on task benchmark/t2 is not carried out yet: the \
         health check of the task did not pass, and cladenc tries again in a new container \
         after 0 seconds, retry 1 of 2\n",
        " in a new container after 0 seconds, retry 2 of 2\n",
    ]
    .map(|retry| said.contains(retry));
    assert_eq!(retries, [true, true]);
}

#[test]
fn cladenc_start_keeps_no_result_of_an_attempt_whose_harness_was_never_started() {
    let (directory, _) = &*WITH_A_TASK_THAT_IS_NEVER_READY;
    let kept = ["attempts", "holdout"].map(|kept| {
        directory
            .path()
            .join("outputs")
            .join(kept)
            .join("0/benchmark/t2")
            .exists()
    });
    assert_eq!(kept, [false, false]);
}

#[test]
fn cladenc_start_keeps_what_the_health_check_printed_where_it_says() {
    let (directory, _) = &*WITH_A_TASK_THAT_IS_NEVER_READY;
    assert_eq!(
        fs::read_to_string(
            directory
                .path()
                .join("outputs/running/attempt-0-benchmark+t2/health-check/stdout")
        )
        .unwrap(),
        "not ready\n"
    );
}

#[test]
fn cladenc_resume_carries_out_again_an_attempt_whose_harness_was_never_started() {
    assert!(
        said(&RESUMED_ONCE_THE_TASK_IS_READY)
            .contains("cladenc: started again the attempt of harness 0 on task benchmark/t2\n")
    );
}

#[test]
fn cladenc_resume_ends_a_directed_evolution_whose_task_is_ready_at_last_with_exit_status_0() {
    assert_eq!(RESUMED_ONCE_THE_TASK_IS_READY.status.code(), Some(0));
}
