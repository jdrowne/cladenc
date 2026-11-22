use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serde_json::Value;
use tempfile::TempDir;

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).stderr(Stdio::null());
    start
}

fn status(directory: &Path, json: Option<&str>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("status")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .args(json)
        .output()
        .unwrap()
}

fn stopped_before_its_end() -> TempDir {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit 125").status().unwrap();
    directory
}

fn lines_of(directory: &Path) -> Vec<String> {
    String::from_utf8(status(directory, None).stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn object_of(directory: &Path) -> Value {
    serde_json::from_slice(&status(directory, Some("--json")).stdout).unwrap()
}

#[test]
fn status_exits_with_0_on_a_directed_evolution_that_stopped_before_its_end() {
    let directory = stopped_before_its_end();
    assert_eq!(status(directory.path(), None).status.code(), Some(0));
}

#[test]
fn status_prints_nothing_on_standard_error_of_a_directed_evolution_that_stopped_before_its_end() {
    let directory = stopped_before_its_end();
    assert_eq!(status(directory.path(), None).stderr, Vec::<u8>::new());
}

#[test]
fn status_says_first_that_the_directed_evolution_has_not_ended() {
    let directory = stopped_before_its_end();
    assert_eq!(
        lines_of(directory.path())[0],
        "the directed evolution has not ended: what follows is its state so far"
    );
}

#[test]
fn status_names_the_harness_that_is_ahead_so_far() {
    let directory = stopped_before_its_end();
    assert_eq!(
        lines_of(directory.path())[2],
        "started 0 of 1 procreation, harness ahead so far 0"
    );
}

#[test]
fn status_prints_a_row_for_the_primogenitor_under_the_header_of_the_table() {
    let directory = stopped_before_its_end();
    assert!(lines_of(directory.path())[9].starts_with("0   -           -           created  "));
}

#[test]
fn with_json_status_prints_an_object_that_says_that_the_directed_evolution_has_not_ended() {
    let directory = stopped_before_its_end();
    assert_eq!(object_of(directory.path())["ended"], Value::Bool(false));
}

#[test]
fn with_json_status_names_the_harness_that_is_ahead_so_far() {
    let directory = stopped_before_its_end();
    assert_eq!(
        object_of(directory.path())["harness_ahead_so_far"].as_u64(),
        Some(0)
    );
}

#[test]
fn with_json_status_names_no_best_harness() {
    let directory = stopped_before_its_end();
    assert!(object_of(directory.path())["best_harness"].is_null());
}

#[test]
fn status_leaves_the_journal_unchanged() {
    let directory = stopped_before_its_end();
    let journal = directory.path().join("outputs/journal.jsonl");
    let before = fs::read(&journal).unwrap();
    status(directory.path(), None);
    assert_eq!(fs::read(&journal).unwrap(), before);
}

#[test]
fn status_exits_with_0_on_a_directed_evolution_that_is_still_going() {
    let directory = test_inputs::temporary_directory();
    let mut going = start(directory.path(), "hostname; sleep 300")
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("outputs/running/attempt-0-benchmark+t1/stdout");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    let status = status(directory.path(), None).status.code();
    kill(Pid::from_raw(going.id().cast_signed()), Signal::SIGINT).unwrap();
    going.wait().unwrap();
    assert_eq!(status, Some(0));
}
