use std::fs;
use std::path::PathBuf;

use cladenc::genealogy::{EvaluationOutcome, MutationOutcome};
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn with_a_mutation_discarded_after_it_ended() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_mutation(1, 0).unwrap();
    workspace.end_mutation(1, MutationOutcome::Created).unwrap();
    workspace.discard_mutation(1).unwrap();
    (directory, workspace)
}

fn with_an_evaluation_discarded_after_it_ended() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_evaluation(0, "t1").unwrap();
    workspace
        .end_evaluation(0, "t1", EvaluationOutcome::Success)
        .unwrap();
    workspace.discard_evaluation(0, "t1").unwrap();
    (directory, workspace)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn discarding_a_mutation_that_is_running_leaves_nothing_running() {
    let (_directory, workspace) = made();
    workspace.start_mutation(1, 0).unwrap();
    workspace.discard_mutation(1).unwrap();
    assert!(!workspace.running_mutation(1).exists());
}

#[test]
fn discarding_a_mutation_that_ended_leaves_no_harness() {
    let (_directory, workspace) = with_a_mutation_discarded_after_it_ended();
    assert!(!workspace.harness(1).exists());
}

#[test]
fn discarding_a_mutation_that_ended_leaves_no_directory_of_the_mutation() {
    let (_directory, workspace) = with_a_mutation_discarded_after_it_ended();
    assert!(!workspace.mutation(1).exists());
}

#[test]
fn discarding_a_mutation_that_ended_leaves_no_directory_for_the_evaluations_of_its_child() {
    let (_directory, workspace) = with_a_mutation_discarded_after_it_ended();
    assert!(!workspace.evaluations(1).exists());
}

#[test]
fn discarding_a_mutation_leaves_the_parent_as_it_was() {
    let (_directory, workspace) = with_a_mutation_discarded_after_it_ended();
    assert!(workspace.harness(0).join("harness.sh").is_file());
}

#[test]
fn discarding_an_evaluation_that_is_running_leaves_nothing_running() {
    let (_directory, workspace) = made();
    workspace.start_evaluation(0, "t1").unwrap();
    workspace.discard_evaluation(0, "t1").unwrap();
    assert!(!workspace.running_evaluation(0, "t1").exists());
}

#[test]
fn discarding_an_evaluation_that_ended_leaves_no_directory_of_the_evaluation() {
    let (_directory, workspace) = with_an_evaluation_discarded_after_it_ended();
    assert!(!workspace.evaluation(0, "t1").exists());
}

#[test]
fn discarding_an_evaluation_keeps_the_directory_for_the_evaluations_of_its_harness() {
    let (_directory, workspace) = with_an_evaluation_discarded_after_it_ended();
    assert_eq!(entries(workspace.evaluations(0)), 0);
}

#[test]
fn discarding_a_job_that_left_nothing_is_not_an_error() {
    let (_directory, workspace) = made();
    assert!(workspace.discard_mutation(1).is_ok());
}
