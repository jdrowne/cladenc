use std::fs;
use std::path::Path;

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::AttemptResult;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const SHOWN: &str = "benchmark/t1";
const HELD_OUT: &str = "benchmark/t2";

fn holding_out_one_task(path: &Path) -> OutputsDirectory {
    OutputsDirectory::at(path).holding_out(&[HELD_OUT.to_owned()])
}

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor)
            .unwrap()
            .holding_out(&[HELD_OUT.to_owned()]);
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn attempted(tasks: &[&str]) -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    for task in tasks {
        outputs_directory.start_attempt(0, task).unwrap();
        outputs_directory
            .end_attempt(0, task, AttemptResult::Completed)
            .unwrap();
    }
    (directory, outputs_directory)
}

fn with_the_attempt_at_the_holdout_task_discarded() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = attempted(&[SHOWN, HELD_OUT]);
    outputs_directory.discard_attempt(0, HELD_OUT).unwrap();
    (directory, outputs_directory)
}

fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn the_attempt_at_a_holdout_task_is_kept_under_holdout() {
    assert_eq!(
        holding_out_one_task(Path::new("/outputs")).attempt(3, HELD_OUT),
        Path::new("/outputs/holdout/3/benchmark/t2")
    );
}

#[test]
fn the_attempt_at_a_task_that_is_not_held_out_is_kept_under_attempts() {
    assert_eq!(
        holding_out_one_task(Path::new("/outputs")).attempt(3, SHOWN),
        Path::new("/outputs/attempts/3/benchmark/t1")
    );
}

#[test]
fn an_outputs_directory_told_of_no_holdout_task_keeps_every_attempt_under_attempts() {
    assert_eq!(
        OutputsDirectory::at(Path::new("/outputs")).attempt(3, HELD_OUT),
        Path::new("/outputs/attempts/3/benchmark/t2")
    );
}

#[test]
fn the_attempts_of_a_harness_at_the_holdout_tasks_are_kept_in_a_directory_named_after_it() {
    assert_eq!(
        holding_out_one_task(Path::new("/outputs")).holdout(3),
        Path::new("/outputs/holdout/3")
    );
}

#[test]
fn the_directory_a_progenitor_is_given_is_the_one_of_its_attempts_that_are_not_held_out() {
    assert_eq!(
        holding_out_one_task(Path::new("/outputs")).attempts(3),
        Path::new("/outputs/attempts/3")
    );
}

#[test]
fn an_outputs_directory_told_of_its_holdout_tasks_is_at_the_same_path() {
    assert_eq!(
        holding_out_one_task(Path::new("/outputs")).path(),
        Path::new("/outputs")
    );
}

#[test]
fn a_new_outputs_directory_holds_no_attempt_at_a_holdout_task() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        names_in(&outputs_directory.path().join("holdout")),
        Vec::<String>::new()
    );
}

#[test]
fn a_started_attempt_at_a_holdout_task_holds_the_specification_of_its_task() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_attempt(0, HELD_OUT).unwrap();
    assert_eq!(
        fs::read_to_string(
            outputs_directory
                .running_attempt(0, HELD_OUT)
                .join("specification")
        )
        .unwrap(),
        "Finish the work of t2.\n"
    );
}

#[test]
fn an_attempt_at_a_holdout_task_that_ended_is_kept_under_holdout_with_its_result() {
    let (_directory, outputs_directory) = attempted(&[HELD_OUT]);
    assert_eq!(
        fs::read_to_string(
            outputs_directory
                .path()
                .join("holdout/0/benchmark/t2/result")
        )
        .unwrap(),
        "completed\n"
    );
}

#[test]
fn an_attempt_at_a_holdout_task_leaves_nothing_in_the_directory_a_progenitor_is_given() {
    let (_directory, outputs_directory) = attempted(&[HELD_OUT]);
    assert_eq!(
        names_in(&outputs_directory.attempts(0)),
        Vec::<String>::new()
    );
}

#[test]
fn the_directory_a_progenitor_is_given_holds_the_tasks_that_are_not_held_out_and_no_other() {
    let (_directory, outputs_directory) = attempted(&[SHOWN, HELD_OUT]);
    assert_eq!(
        names_in(&outputs_directory.attempts(0).join("benchmark")),
        ["t1"]
    );
}

#[test]
fn the_attempts_at_the_holdout_tasks_are_kept_apart_by_benchmark_and_task() {
    let (_directory, outputs_directory) = attempted(&[SHOWN, HELD_OUT]);
    assert_eq!(
        names_in(&outputs_directory.holdout(0).join("benchmark")),
        ["t2"]
    );
}

#[test]
fn discarding_an_attempt_at_a_holdout_task_that_ended_leaves_no_directory_of_the_attempt() {
    let (_directory, outputs_directory) = with_the_attempt_at_the_holdout_task_discarded();
    assert!(
        !outputs_directory
            .path()
            .join("holdout/0/benchmark/t2")
            .exists()
    );
}

#[test]
fn discarding_an_attempt_at_a_holdout_task_leaves_the_attempts_that_are_not_held_out() {
    let (_directory, outputs_directory) = with_the_attempt_at_the_holdout_task_discarded();
    assert!(outputs_directory.attempt(0, SHOWN).join("result").is_file());
}

#[test]
fn a_discarded_attempt_at_a_holdout_task_can_be_started_again() {
    let (_directory, outputs_directory) = with_the_attempt_at_the_holdout_task_discarded();
    assert!(outputs_directory.start_attempt(0, HELD_OUT).is_ok());
}
