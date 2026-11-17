use std::fs;
use std::path::PathBuf;

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::{AttemptResult, ProcreationOutcome};
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, workspace)
}

fn with_a_procreation_discarded_after_it_ended() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_procreation(1, 0).unwrap();
    workspace
        .end_procreation(1, ProcreationOutcome::Created)
        .unwrap();
    workspace.discard_procreation(1).unwrap();
    (directory, workspace)
}

fn with_an_attempt_discarded_after_it_ended() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_attempt(0, "benchmark/t1").unwrap();
    workspace
        .end_attempt(0, "benchmark/t1", AttemptResult::Completed)
        .unwrap();
    workspace.discard_attempt(0, "benchmark/t1").unwrap();
    (directory, workspace)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn discarding_a_procreation_that_is_running_leaves_nothing_running() {
    let (_directory, workspace) = made();
    workspace.start_procreation(1, 0).unwrap();
    workspace.discard_procreation(1).unwrap();
    assert!(!workspace.running_procreation(1).exists());
}

#[test]
fn a_discarded_procreation_can_be_started_again() {
    let (_directory, workspace) = made();
    workspace.start_procreation(1, 0).unwrap();
    workspace.discard_procreation(1).unwrap();
    assert!(workspace.start_procreation(1, 0).is_ok());
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_harness() {
    let (_directory, workspace) = with_a_procreation_discarded_after_it_ended();
    assert!(!workspace.harness(1).exists());
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_directory_of_the_procreation() {
    let (_directory, workspace) = with_a_procreation_discarded_after_it_ended();
    assert!(!workspace.procreation(1).exists());
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_directory_for_the_attempts_of_its_child() {
    let (_directory, workspace) = with_a_procreation_discarded_after_it_ended();
    assert!(!workspace.attempts(1).exists());
}

#[test]
fn discarding_a_procreation_leaves_no_build_of_its_child() {
    let (_directory, workspace) = made();
    workspace.start_procreation(1, 0).unwrap();
    workspace.start_build(1).unwrap();
    workspace.discard_procreation(1).unwrap();
    assert!(!workspace.build(1).exists());
}

#[test]
fn discarding_a_procreation_leaves_the_build_of_another_harness_as_it_was() {
    let (_directory, workspace) = made();
    workspace.start_build(0).unwrap();
    workspace.start_procreation(1, 0).unwrap();
    workspace.discard_procreation(1).unwrap();
    assert!(workspace.build(0).is_dir());
}

#[test]
fn discarding_a_procreation_leaves_the_progenitor_as_it_was() {
    let (_directory, workspace) = with_a_procreation_discarded_after_it_ended();
    assert!(workspace.harness(0).join("harness.sh").is_file());
}

#[test]
fn discarding_an_attempt_that_is_running_leaves_nothing_running() {
    let (_directory, workspace) = made();
    workspace.start_attempt(0, "benchmark/t1").unwrap();
    workspace.discard_attempt(0, "benchmark/t1").unwrap();
    assert!(!workspace.running_attempt(0, "benchmark/t1").exists());
}

#[test]
fn discarding_an_attempt_that_ended_leaves_no_directory_of_the_attempt() {
    let (_directory, workspace) = with_an_attempt_discarded_after_it_ended();
    assert!(!workspace.attempt(0, "benchmark/t1").exists());
}

#[test]
fn discarding_an_attempt_keeps_the_directory_for_the_attempts_of_its_harness() {
    let (_directory, workspace) = with_an_attempt_discarded_after_it_ended();
    assert!(workspace.attempts(0).is_dir());
}

#[test]
fn discarding_an_attempt_that_ended_leaves_no_attempt_in_the_directory_of_its_benchmark() {
    let (_directory, workspace) = with_an_attempt_discarded_after_it_ended();
    assert_eq!(entries(workspace.attempts(0).join("benchmark")), 0);
}

#[test]
fn a_discarded_attempt_can_be_started_again() {
    let (_directory, workspace) = with_an_attempt_discarded_after_it_ended();
    assert!(workspace.start_attempt(0, "benchmark/t1").is_ok());
}

#[test]
fn discarding_a_procreation_that_left_nothing_is_not_an_error() {
    let (_directory, workspace) = made();
    assert!(workspace.discard_procreation(1).is_ok());
}
