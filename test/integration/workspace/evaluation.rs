use std::fs;
use std::path::PathBuf;

use cladenc::genealogy::EvaluationOutcome;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn started() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    workspace.start_evaluation(0, "t1").unwrap();
    (directory, workspace)
}

fn ended(outcome: EvaluationOutcome) -> (TempDir, Workspace) {
    let (directory, workspace) = started();
    workspace.end_evaluation(0, "t1", outcome).unwrap();
    (directory, workspace)
}

fn outcome_written(outcome: EvaluationOutcome) -> String {
    let (_directory, workspace) = ended(outcome);
    fs::read_to_string(workspace.evaluation(0, "t1").join("outcome")).unwrap()
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn a_started_evaluation_holds_a_copy_of_its_harness() {
    let (_directory, workspace) = started();
    assert_eq!(
        fs::read_to_string(
            workspace
                .running_evaluation(0, "t1")
                .join("harness/harness.sh")
        )
        .unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_evaluation_holds_an_empty_output_directory() {
    let (_directory, workspace) = started();
    assert_eq!(
        entries(workspace.running_evaluation(0, "t1").join("output")),
        0
    );
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
    assert!(!workspace.evaluation(0, "t1").join("harness").exists());
}

#[test]
fn an_ended_evaluation_keeps_its_output() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(workspace.evaluation(0, "t1").join("output").is_dir());
}

#[test]
fn an_ended_evaluation_leaves_nothing_running() {
    let (_directory, workspace) = ended(EvaluationOutcome::Success);
    assert!(!workspace.running_evaluation(0, "t1").exists());
}
