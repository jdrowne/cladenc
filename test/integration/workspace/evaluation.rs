use std::fs;
use std::path::{Path, PathBuf};

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::EvaluationOutcome;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, workspace)
}

fn started() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_evaluation(0, "benchmark/t1").unwrap();
    (directory, workspace)
}

fn ended(outcome: EvaluationOutcome) -> (TempDir, Workspace) {
    let (directory, workspace) = started();
    workspace
        .end_evaluation(0, "benchmark/t1", outcome)
        .unwrap();
    (directory, workspace)
}

fn outcome_written(outcome: EvaluationOutcome) -> String {
    let (_directory, workspace) = ended(outcome);
    fs::read_to_string(workspace.evaluation(0, "benchmark/t1").join("outcome")).unwrap()
}

fn held_while_running(file: &str) -> String {
    let (_directory, workspace) = started();
    fs::read_to_string(workspace.running_evaluation(0, "benchmark/t1").join(file)).unwrap()
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn a_started_evaluation_holds_a_copy_of_its_harness() {
    assert_eq!(
        held_while_running("harness/harness.sh"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_evaluation_holds_a_copy_of_the_files_of_its_task_to_work_on() {
    assert_eq!(held_while_running("work/work.txt"), "to do\n");
}

#[test]
fn a_started_evaluation_holds_the_statement_of_its_task() {
    assert_eq!(held_while_running("statement"), "Finish the work of t1.\n");
}

#[test]
fn a_started_evaluation_holds_an_empty_directory_for_what_the_check_prints() {
    let (_directory, workspace) = started();
    assert_eq!(
        entries(
            workspace
                .running_evaluation(0, "benchmark/t1")
                .join("check")
        ),
        0
    );
}

#[test]
fn a_started_evaluation_holds_no_copy_of_the_check() {
    let (_directory, workspace) = started();
    assert!(
        !workspace
            .running_evaluation(0, "benchmark/t1")
            .join("work/check")
            .exists()
    );
}

#[test]
fn a_started_evaluation_has_an_empty_directory_for_its_benchmark_among_those_of_its_harness() {
    let (_directory, workspace) = started();
    assert_eq!(entries(workspace.evaluations(0).join("benchmark")), 0);
}

#[test]
fn an_evaluation_on_a_task_the_run_directory_does_not_hold_cannot_be_started() {
    let (_directory, workspace) = made();
    assert!(workspace.start_evaluation(0, "benchmark/t9").is_err());
}

#[test]
fn an_evaluation_that_succeeded_leaves_the_word_success_as_its_outcome() {
    assert_eq!(outcome_written(EvaluationOutcome::Success), "success\n");
}

#[test]
fn an_evaluation_that_failed_leaves_the_word_failure_as_its_outcome() {
    assert_eq!(outcome_written(EvaluationOutcome::Failure), "failure\n");
}

#[test]
fn an_evaluation_that_timed_out_leaves_the_word_timed_out_as_its_outcome() {
    assert_eq!(outcome_written(EvaluationOutcome::TimedOut), "timed_out\n");
}

#[test]
fn an_ended_evaluation_keeps_no_copy_of_its_harness() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(
        !workspace
            .evaluation(0, "benchmark/t1")
            .join("harness")
            .exists()
    );
}

#[test]
fn an_ended_evaluation_keeps_the_work() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(
        workspace
            .evaluation(0, "benchmark/t1")
            .join("work/work.txt")
            .is_file()
    );
}

#[test]
fn an_ended_evaluation_keeps_the_statement_of_its_task() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(
        workspace
            .evaluation(0, "benchmark/t1")
            .join("statement")
            .is_file()
    );
}

#[test]
fn an_ended_evaluation_leaves_nothing_running() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(!workspace.running_evaluation(0, "benchmark/t1").exists());
}

#[test]
fn the_directory_of_an_ended_evaluation_is_named_after_its_harness_its_benchmark_and_its_task() {
    assert_eq!(
        Workspace::at(Path::new("/run")).evaluation(3, "parsing/fix-the-parser"),
        Path::new("/run/evaluations/3/parsing/fix-the-parser")
    );
}

#[test]
fn the_directory_of_a_running_evaluation_names_its_benchmark_and_its_task_in_one_name() {
    assert_eq!(
        Workspace::at(Path::new("/run")).running_evaluation(3, "parsing/fix-the-parser"),
        Path::new("/run/running/evaluation-3-parsing+fix-the-parser")
    );
}

#[test]
fn an_evaluation_that_ends_after_another_on_a_task_of_the_same_benchmark_is_kept_beside_it() {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    for task in ["benchmark/t1", "benchmark/t2"] {
        workspace.start_evaluation(0, task).unwrap();
        workspace
            .end_evaluation(0, task, EvaluationOutcome::Success)
            .unwrap();
    }
    assert_eq!(entries(workspace.evaluations(0).join("benchmark")), 2);
}
