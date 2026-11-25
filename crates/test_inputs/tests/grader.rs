use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use test_inputs::{benchmark, kept_grader, temporary_directory};

fn as_made(task: &str) -> String {
    format!(
        "#!/bin/sh\nif grep -q -x -e {task} solved; then echo 1; else echo 0; fi > \
         /logs/verifier/reward.txt\n"
    )
}

fn read(file: &Path) -> String {
    fs::read_to_string(file).unwrap()
}

fn kept_with_the_grader_of_t1_written_anew() -> (TempDir, PathBuf) {
    let directory = temporary_directory();
    let outputs = directory.path().join("outputs");
    let kept = benchmark(&outputs.join("benchmarks"), &["t1", "t2"]);
    kept_grader(&outputs, "t1", "exit 3");
    (directory, kept)
}

#[test]
fn the_grader_kept_in_an_outputs_directory_is_written_anew_with_the_command() {
    let (_directory, kept) = kept_with_the_grader_of_t1_written_anew();
    assert_eq!(read(&kept.join("t1/tests/test.sh")), "#!/bin/sh\nexit 3\n");
}

#[test]
fn a_grader_written_anew_can_be_run_as_before() {
    let (_directory, kept) = kept_with_the_grader_of_t1_written_anew();
    assert_eq!(
        fs::metadata(kept.join("t1/tests/test.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o750
    );
}

#[test]
fn the_grader_of_another_task_is_left_as_it_was() {
    let (_directory, kept) = kept_with_the_grader_of_t1_written_anew();
    assert_eq!(read(&kept.join("t2/tests/test.sh")), as_made("t2"));
}

#[test]
fn the_grader_a_person_brought_is_left_as_it_was_when_the_kept_one_is_written_anew() {
    let directory = temporary_directory();
    let outputs = directory.path().join("outputs");
    let brought = benchmark(directory.path(), &["t1"]);
    benchmark(&outputs.join("benchmarks"), &["t1"]);
    kept_grader(&outputs, "t1", "exit 3");
    assert_eq!(read(&brought.join("t1/tests/test.sh")), as_made("t1"));
}
