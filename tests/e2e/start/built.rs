use std::fs;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

fn ran(build_command: &str) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let options = [
        (
            "--primogenitor",
            test_inputs::primogenitor(directory.path()),
        ),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-build-command", build_command.into()),
        ("--harness-run-command", "touch changed".into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, output)
}

static BUILT: LazyLock<(TempDir, Output)> = LazyLock::new(|| ran("echo built >> built"));

static NOT_BUILT: LazyLock<(TempDir, Output)> = LazyLock::new(|| ran("exit 3"));

fn left_in(directory: &TempDir, file: &str) -> String {
    fs::read_to_string(directory.path().join("outputs").join(file)).unwrap()
}

#[test]
fn a_run_given_a_build_command_builds_the_primogenitor() {
    assert_eq!(left_in(&BUILT.0, "harnesses/0/built"), "built\n");
}

#[test]
fn a_run_given_a_build_command_builds_a_child_after_its_progenitor() {
    assert_eq!(left_in(&BUILT.0, "harnesses/1/built"), "built\nbuilt\n");
}

#[test]
fn a_run_keeps_what_the_build_of_a_harness_wrote_under_builds() {
    let (directory, _output) = ran("echo making");
    assert_eq!(left_in(&directory, "builds/1/stdout"), "making\n");
}

#[test]
fn a_run_whose_primogenitor_does_not_build_exits_with_1() {
    let (_directory, output) = &*NOT_BUILT;
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_run_whose_primogenitor_does_not_build_says_how_the_build_ended() {
    let (_directory, output) = &*NOT_BUILT;
    assert_eq!(
        str::from_utf8(&output.stderr).unwrap().lines().last(),
        Some(
            "cladenc: the directed evolution stopped before its end: \
             the build of the primogenitor exited with status 3"
        )
    );
}
