use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::file_system::tests::double::Memory;
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&directory, &benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn started() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    outputs_directory.start_attempt(0, "benchmark/t1").unwrap();
    (directory, outputs_directory)
}

fn ended(result: AttemptResult) -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    outputs_directory
        .end_attempt(0, "benchmark/t1", result)
        .unwrap();
    (directory, outputs_directory)
}

fn result_written(result: AttemptResult) -> String {
    let (directory, outputs_directory) = ended(result);
    directory
        .read_to_string(outputs_directory.attempt(0, "benchmark/t1").join("result"))
        .unwrap()
}

fn held_while_running(file: &str) -> String {
    let (directory, outputs_directory) = started();
    directory
        .read_to_string(
            outputs_directory
                .running_attempt(0, "benchmark/t1")
                .join(file),
        )
        .unwrap()
}

fn entries(files: &Memory, path: PathBuf) -> usize {
    files.names_in(path).len()
}

#[test]
fn a_started_attempt_holds_the_specification_of_its_task() {
    assert_eq!(
        held_while_running("specification"),
        "Finish the work of t1.\n"
    );
}

#[test]
fn a_started_attempt_holds_a_directory_for_the_verifier_with_nothing_in_it_but_its_logs() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        entries(
            &directory,
            outputs_directory
                .running_attempt(0, "benchmark/t1")
                .join("verifier")
        ),
        1
    );
}

#[test]
fn a_started_attempt_holds_an_empty_directory_for_what_the_verifier_leaves_in_logs_verifier() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        entries(
            &directory,
            outputs_directory
                .running_attempt(0, "benchmark/t1")
                .join("verifier/logs/verifier")
        ),
        0
    );
}

#[test]
fn a_started_attempt_holds_nothing_but_the_specification_and_a_directory_for_the_verifier() {
    let (directory, outputs_directory) = started();
    let names = directory.names_in(outputs_directory.running_attempt(0, "benchmark/t1"));
    assert_eq!(names, ["specification", "verifier"]);
}

#[test]
fn a_started_attempt_has_an_empty_directory_for_its_benchmark_among_those_of_its_harness() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        entries(&directory, outputs_directory.attempts(0).join("benchmark")),
        0
    );
}

#[test]
fn an_attempt_of_a_harness_the_outputs_directory_does_not_hold_cannot_be_started() {
    let (_directory, outputs_directory) = made();
    assert!(outputs_directory.start_attempt(7, "benchmark/t1").is_err());
}

#[test]
fn an_attempt_of_a_harness_the_outputs_directory_does_not_hold_leaves_nothing_running() {
    let (directory, outputs_directory) = made();
    let _ = outputs_directory.start_attempt(7, "benchmark/t1");
    assert!(!directory.exists(outputs_directory.running_attempt(7, "benchmark/t1")));
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
    let (directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(!directory.exists(outputs_directory.attempt(0, "benchmark/t1").join("harness")));
}

#[test]
fn an_ended_attempt_keeps_what_was_left_in_the_directory_of_the_running_attempt() {
    let (directory, outputs_directory) = started();
    let running = outputs_directory.running_attempt(0, "benchmark/t1");
    directory.create_dir(running.join("work")).unwrap();
    directory
        .write(running.join("work/left"), "done\n")
        .unwrap();
    outputs_directory
        .end_attempt(0, "benchmark/t1", AttemptResult::Completed)
        .unwrap();
    assert_eq!(
        directory
            .read_to_string(
                outputs_directory
                    .attempt(0, "benchmark/t1")
                    .join("work/left")
            )
            .unwrap(),
        "done\n"
    );
}

#[test]
fn an_ended_attempt_keeps_the_specification_of_its_task() {
    let (directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(
        directory.is_file(
            outputs_directory
                .attempt(0, "benchmark/t1")
                .join("specification")
        )
    );
}

#[test]
fn an_ended_attempt_leaves_nothing_running() {
    let (directory, outputs_directory) = ended(AttemptResult::Completed);
    assert!(!directory.exists(outputs_directory.running_attempt(0, "benchmark/t1")));
}

#[test]
fn the_directory_of_an_ended_attempt_is_named_after_its_harness_its_benchmark_and_its_task() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
            .attempt(3, "parsing/fix-the-parser"),
        Path::new("/outputs/attempts/3/parsing/fix-the-parser")
    );
}

#[test]
fn the_directory_of_a_running_attempt_names_its_benchmark_and_its_task_in_one_name() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
            .running_attempt(3, "parsing/fix-the-parser"),
        Path::new("/outputs/running/attempt-3-parsing+fix-the-parser")
    );
}

#[test]
fn an_attempt_that_ends_after_another_on_a_task_of_the_same_benchmark_is_kept_beside_it() {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&directory, &benchmark).unwrap())
        .unwrap();
    for task in ["benchmark/t1", "benchmark/t2"] {
        outputs_directory.start_attempt(0, task).unwrap();
        outputs_directory
            .end_attempt(0, task, AttemptResult::Completed)
            .unwrap();
    }
    assert_eq!(
        entries(&directory, outputs_directory.attempts(0).join("benchmark")),
        2
    );
}
