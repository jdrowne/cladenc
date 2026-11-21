use std::fs;
use std::path::{Path, PathBuf};

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::AttemptResult;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn started() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    outputs_directory.start_attempt(0, "benchmark/t1").unwrap();
    (directory, outputs_directory)
}

fn ended(result: AttemptResult) -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    outputs_directory
        .end_attempt(0, "benchmark/t1", result)
        .unwrap();
    (directory, outputs_directory)
}

fn result_written(result: AttemptResult) -> String {
    let (_directory, outputs_directory) = ended(result);
    fs::read_to_string(outputs_directory.attempt(0, "benchmark/t1").join("result")).unwrap()
}

fn held_while_running(file: &str) -> String {
    let (_directory, outputs_directory) = started();
    fs::read_to_string(
        outputs_directory
            .running_attempt(0, "benchmark/t1")
            .join(file),
    )
    .unwrap()
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn a_started_attempt_holds_a_copy_of_its_harness() {
    assert_eq!(
        held_while_running("harness/harness.sh"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_attempt_holds_a_copy_of_the_workspace_of_its_task_to_work_on() {
    assert_eq!(held_while_running("work/work.txt"), "to do\n");
}

#[test]
fn a_started_attempt_holds_the_specification_of_its_task() {
    assert_eq!(
        held_while_running("specification"),
        "Finish the work of t1.\n"
    );
}

#[test]
fn a_started_attempt_holds_a_directory_for_the_grader_with_nothing_in_it_but_its_logs() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        entries(
            outputs_directory
                .running_attempt(0, "benchmark/t1")
                .join("grader")
        ),
        1
    );
}

#[test]
fn a_started_attempt_holds_an_empty_directory_for_what_the_grader_leaves_in_logs_verifier() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        entries(
            outputs_directory
                .running_attempt(0, "benchmark/t1")
                .join("grader/logs/verifier")
        ),
        0
    );
}

#[test]
fn a_started_attempt_holds_no_copy_of_the_tests_of_the_task() {
    let (_directory, outputs_directory) = started();
    assert!(
        !outputs_directory
            .running_attempt(0, "benchmark/t1")
            .join("work/tests")
            .exists()
    );
}

#[test]
fn a_started_attempt_has_an_empty_directory_for_its_benchmark_among_those_of_its_harness() {
    let (_directory, outputs_directory) = started();
    assert_eq!(entries(outputs_directory.attempts(0).join("benchmark")), 0);
}

#[test]
fn an_attempt_on_a_task_the_outputs_directory_does_not_hold_cannot_be_started() {
    let (_directory, outputs_directory) = made();
    assert!(outputs_directory.start_attempt(0, "benchmark/t9").is_err());
}

#[test]
fn a_completed_attempt_leaves_the_word_completed_as_its_result() {
    assert_eq!(result_written(AttemptResult::Completed), "completed\n");
}

#[test]
fn an_attempt_not_completed_leaves_the_word_not_completed_as_its_result() {
    assert_eq!(
        result_written(AttemptResult::NotCompleted),
        "not_completed\n"
    );
}

#[test]
fn a_harness_whose_attempt_timed_out_leaves_the_word_timed_out_as_its_result() {
    assert_eq!(result_written(AttemptResult::TimedOut), "timed_out\n");
}

#[test]
fn an_ended_attempt_keeps_no_copy_of_its_harness() {
    let (_directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(
        !outputs_directory
            .attempt(0, "benchmark/t1")
            .join("harness")
            .exists()
    );
}

#[test]
fn an_ended_attempt_keeps_the_work() {
    let (_directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(
        outputs_directory
            .attempt(0, "benchmark/t1")
            .join("work/work.txt")
            .is_file()
    );
}

#[test]
fn an_ended_attempt_keeps_the_specification_of_its_task() {
    let (_directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(
        outputs_directory
            .attempt(0, "benchmark/t1")
            .join("specification")
            .is_file()
    );
}

#[test]
fn an_ended_attempt_leaves_nothing_running() {
    let (_directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(
        !outputs_directory
            .running_attempt(0, "benchmark/t1")
            .exists()
    );
}

#[test]
fn the_directory_of_an_ended_attempt_is_named_after_its_harness_its_benchmark_and_its_task() {
    assert_eq!(
        OutputsDirectory::at(Path::new("/outputs")).attempt(3, "parsing/fix-the-parser"),
        Path::new("/outputs/attempts/3/parsing/fix-the-parser")
    );
}

#[test]
fn the_directory_of_a_running_attempt_names_its_benchmark_and_its_task_in_one_name() {
    assert_eq!(
        OutputsDirectory::at(Path::new("/outputs")).running_attempt(3, "parsing/fix-the-parser"),
        Path::new("/outputs/running/attempt-3-parsing+fix-the-parser")
    );
}

#[test]
fn an_attempt_that_ends_after_another_on_a_task_of_the_same_benchmark_is_kept_beside_it() {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    for task in ["benchmark/t1", "benchmark/t2"] {
        outputs_directory.start_attempt(0, task).unwrap();
        outputs_directory
            .end_attempt(0, task, AttemptResult::Completed)
            .unwrap();
    }
    assert_eq!(entries(outputs_directory.attempts(0).join("benchmark")), 2);
}
