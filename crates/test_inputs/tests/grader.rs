use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use test_inputs::{benchmark, grader, kept_grader, temporary_directory};

fn read(file: &Path) -> String {
    fs::read_to_string(file).unwrap()
}

#[test]
fn the_grader_of_a_task_is_written_anew_with_the_command() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1"]);
    grader(directory.path(), "t1", "exit 3");
    assert_eq!(read(&made.join("t1/tests/test.sh")), "#!/bin/sh\nexit 3\n");
}

#[test]
fn a_grader_written_anew_can_be_run_as_before() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1"]);
    grader(directory.path(), "t1", "exit 3");
    assert_eq!(
        fs::metadata(made.join("t1/tests/test.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o750
    );
}

#[test]
fn the_grader_of_another_task_is_left_as_it_was() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1", "t2"]);
    grader(directory.path(), "t1", "exit 3");
    assert_eq!(
        read(&made.join("t2/tests/test.sh")),
        "#!/bin/sh\nif grep -q -x -e t2 solved; then echo 1; else echo 0; fi \
         > /logs/verifier/reward.txt\n"
    );
}

#[test]
fn the_grader_kept_in_an_outputs_directory_is_written_anew_with_the_command() {
    let directory = temporary_directory();
    let outputs = directory.path().join("outputs");
    let kept = benchmark(&outputs.join("benchmarks"), &["t1"]);
    kept_grader(&outputs, "t1", "exit 3");
    assert_eq!(read(&kept.join("t1/tests/test.sh")), "#!/bin/sh\nexit 3\n");
}

#[test]
fn the_grader_a_person_brought_is_left_as_it_was_when_the_kept_one_is_written_anew() {
    let directory = temporary_directory();
    let outputs = directory.path().join("outputs");
    let brought = benchmark(directory.path(), &["t1"]);
    benchmark(&outputs.join("benchmarks"), &["t1"]);
    kept_grader(&outputs, "t1", "exit 3");
    assert_eq!(
        read(&brought.join("t1/tests/test.sh")),
        "#!/bin/sh\nif grep -q -x -e t1 solved; then echo 1; else echo 0; fi \
         > /logs/verifier/reward.txt\n"
    );
}
