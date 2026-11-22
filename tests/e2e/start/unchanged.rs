use std::process::{Command, Output};

use tempfile::TempDir;

fn ran() -> (TempDir, Output) {
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
        ("--harness-run-command", "exit 0".into()),
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

#[test]
fn a_run_whose_procreation_changed_nothing_exits_with_0() {
    let (_directory, output) = ran();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_run_whose_procreation_changed_nothing_prints_the_directory_of_the_primogenitor() {
    let (directory, output) = ran();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{}\n",
            directory.path().join("outputs/harnesses/0").display()
        )
    );
}

#[test]
fn a_run_says_that_a_procreation_left_the_copy_unchanged_and_created_no_harness() {
    let (directory, output) = ran();
    assert!(String::from_utf8(output.stderr).unwrap().contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 left the copy of the source code of \
         harness 0 unchanged, and created no harness: what the harness printed is kept in {}\n",
        directory.path().join("outputs/procreations/1").display()
    )));
}

#[test]
fn the_report_of_a_run_shows_a_child_that_would_be_the_copy_as_it_was_as_unchanged() {
    let (_directory, output) = ran();
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .last()
            .is_some_and(|row| row.starts_with("1   0           -           unchanged  "))
    );
}

#[test]
fn a_run_benchmarks_no_child_that_would_be_the_copy_as_it_was() {
    let (directory, _output) = ran();
    assert!(!directory.path().join("outputs/attempts/1").exists());
}
