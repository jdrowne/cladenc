use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::file_system::tests::double::Memory;
use crate::genealogy::{AttemptResult, CreationOutcome};
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

fn with_a_procreation_discarded_after_it_ended() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory
        .end_procreation(1, 0, CreationOutcome::Created)
        .unwrap();
    outputs_directory.discard_procreation(1).unwrap();
    (directory, outputs_directory)
}

fn with_an_attempt_discarded_after_it_ended() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    outputs_directory.start_attempt(0, "benchmark/t1").unwrap();
    outputs_directory
        .end_attempt(0, "benchmark/t1", AttemptResult::Completed)
        .unwrap();
    outputs_directory
        .discard_attempt(0, "benchmark/t1")
        .unwrap();
    (directory, outputs_directory)
}

fn entries(files: &Memory, path: PathBuf) -> usize {
    files.names_in(path).len()
}

#[test]
fn discarding_a_procreation_that_is_running_leaves_nothing_running() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory.discard_procreation(1).unwrap();
    assert!(!directory.exists(outputs_directory.running_procreation(1)));
}

#[test]
fn a_discarded_procreation_can_be_started_again() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory.discard_procreation(1).unwrap();
    assert!(outputs_directory.start_procreation(1, 0).is_ok());
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_harness() {
    let (directory, outputs_directory) = with_a_procreation_discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.harness(1)));
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_directory_of_the_procreation() {
    let (directory, outputs_directory) = with_a_procreation_discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.procreation(1)));
}

#[test]
fn discarding_a_procreation_that_ended_leaves_no_directory_for_the_attempts_of_its_child() {
    let (directory, outputs_directory) = with_a_procreation_discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.attempts(1)));
}

#[test]
fn discarding_a_procreation_leaves_no_build_of_its_child() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory.start_build(1).unwrap();
    outputs_directory.discard_procreation(1).unwrap();
    assert!(!directory.exists(outputs_directory.build(1)));
}

#[test]
fn discarding_a_procreation_leaves_the_build_of_another_harness_as_it_was() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_build(0).unwrap();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory.discard_procreation(1).unwrap();
    assert!(directory.is_dir(outputs_directory.build(0)));
}

#[test]
fn discarding_a_procreation_leaves_the_progenitor_as_it_was() {
    let (directory, outputs_directory) = with_a_procreation_discarded_after_it_ended();
    assert!(directory.is_file(outputs_directory.harness(0).join("harness.sh")));
}

#[test]
fn discarding_an_attempt_that_is_running_leaves_nothing_running() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_attempt(0, "benchmark/t1").unwrap();
    outputs_directory
        .discard_attempt(0, "benchmark/t1")
        .unwrap();
    assert!(!directory.exists(outputs_directory.running_attempt(0, "benchmark/t1")));
}

#[test]
fn discarding_an_attempt_that_ended_leaves_no_directory_of_the_attempt() {
    let (directory, outputs_directory) = with_an_attempt_discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.attempt(0, "benchmark/t1")));
}

#[test]
fn discarding_an_attempt_keeps_the_directory_for_the_attempts_of_its_harness() {
    let (directory, outputs_directory) = with_an_attempt_discarded_after_it_ended();
    assert!(directory.is_dir(outputs_directory.attempts(0)));
}

#[test]
fn discarding_an_attempt_that_ended_leaves_no_attempt_in_the_directory_of_its_benchmark() {
    let (directory, outputs_directory) = with_an_attempt_discarded_after_it_ended();
    assert_eq!(
        entries(&directory, outputs_directory.attempts(0).join("benchmark")),
        0
    );
}

#[test]
fn a_discarded_attempt_can_be_started_again() {
    let (_directory, outputs_directory) = with_an_attempt_discarded_after_it_ended();
    assert!(outputs_directory.start_attempt(0, "benchmark/t1").is_ok());
}

#[test]
fn discarding_a_procreation_that_left_nothing_is_not_an_error() {
    let (_directory, outputs_directory) = made();
    assert!(outputs_directory.discard_procreation(1).is_ok());
}
