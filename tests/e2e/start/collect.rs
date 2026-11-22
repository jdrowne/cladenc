use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const PUBLISH_A_REPORT: &str =
    "mkdir -p /logs/artifacts 2> /dev/null && echo found > /logs/artifacts/report.txt; true";
const SAY_COLLECTED: &str = "[[verifier.collect]]\ncommand = \"echo collected\"\n";

fn start(directory: &Path, run_command: &str, task_toml_of_t2: &str) -> Command {
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
        ("--procreation-timeout", "30".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    fs::write(directory.join("benchmark/t2/task.toml"), task_toml_of_t2).unwrap();
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn started(run_command: &str, task_toml_of_t2: &str) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path(), run_command, task_toml_of_t2)
        .output()
        .unwrap();
    (directory, output)
}

fn kept_of_the_attempt_at_t2(directory: &TempDir, file: &str) -> String {
    let attempt = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t2");
    fs::read_to_string(attempt.join(file)).unwrap()
}

fn said(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn what_a_harness_left_in_logs_artifacts_is_kept_with_its_attempt() {
    let (directory, _output) = started(PUBLISH_A_REPORT, "");
    assert_eq!(
        kept_of_the_attempt_at_t2(&directory, "artifacts/logs/artifacts/report.txt"),
        "found\n"
    );
}

#[test]
fn what_a_command_that_a_task_lists_under_collect_printed_is_kept_with_the_attempt() {
    let (directory, _output) = started("exit 0", SAY_COLLECTED);
    assert_eq!(
        kept_of_the_attempt_at_t2(&directory, "collect/1/stdout"),
        "collected\n"
    );
}

#[test]
fn a_task_that_lists_a_command_for_another_container_under_collect_is_reported() {
    assert!(
        said(&started("exit 0", &format!("{SAY_COLLECTED}service = \"api\"\n")).1).ends_with(
            "benchmark: task t2 holds a task.toml that asks for a command in a container other \
             than that of the harness, which cladenc has no way to give\n"
        )
    );
}

#[test]
fn a_task_that_lists_an_entry_with_no_command_under_collect_exits_with_2() {
    let (_directory, output) = started("exit 0", "[[verifier.collect]]\ntimeout_sec = 30\n");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_task_that_lists_an_entry_with_no_command_under_collect_is_reported_as_not_valid() {
    let (_directory, output) = started("exit 0", "[[verifier.collect]]\ntimeout_sec = 30\n");
    assert!(said(&output).ends_with(": missing field `command`\n"));
}
