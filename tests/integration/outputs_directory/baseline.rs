use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use cladenc::genealogy::{CreationOutcome, Experiment};
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const EXPERIMENT: Experiment = Experiment {
    child: 2,
    progenitor: 1,
};

fn with_a_child_of_the_primogenitor() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    outputs_directory.start_procreation(1, 0).unwrap();
    fs::write(
        outputs_directory
            .running_procreation(1)
            .join("child/harness.sh"),
        "#!/bin/sh\necho child\n",
    )
    .unwrap();
    outputs_directory
        .end_procreation(1, 0, CreationOutcome::Created)
        .unwrap();
    (directory, outputs_directory)
}

fn started() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = with_a_child_of_the_primogenitor();
    outputs_directory.start_baseline(3, 1).unwrap();
    (directory, outputs_directory)
}

fn ended(outcome: CreationOutcome) -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    fs::write(
        outputs_directory
            .running_baseline(3)
            .join("singleton-control-group-harness/new.txt"),
        "new",
    )
    .unwrap();
    outputs_directory
        .end_baseline(3, EXPERIMENT, outcome)
        .unwrap();
    (directory, outputs_directory)
}

fn discarded_after_it_ended() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    outputs_directory.discard_baseline(3).unwrap();
    (directory, outputs_directory)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

fn names_in(directory: PathBuf) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn recorded(outcome: CreationOutcome, file: &str) -> String {
    let (_directory, outputs_directory) = ended(outcome);
    fs::read_to_string(outputs_directory.baseline(3).join(file)).unwrap()
}

fn harness_script(harness: PathBuf) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn a_new_outputs_directory_holds_no_baseline() {
    let (_directory, outputs_directory) = with_a_child_of_the_primogenitor();
    assert_eq!(entries(outputs_directory.path().join("baselines")), 0);
}

#[test]
fn a_started_baseline_holds_a_copy_of_the_primogenitor_to_run() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        harness_script(outputs_directory.running_baseline(3).join("primogenitor")),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_baseline_holds_a_copy_of_the_progenitor_to_change() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        harness_script(
            outputs_directory
                .running_baseline(3)
                .join("singleton-control-group-harness")
        ),
        "#!/bin/sh\necho child\n"
    );
}

#[test]
fn a_started_baseline_holds_its_two_copies_and_nothing_else() {
    let (_directory, outputs_directory) = started();
    assert_eq!(entries(outputs_directory.running_baseline(3)), 2);
}

#[test]
fn a_baseline_cannot_be_started_twice() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        outputs_directory.start_baseline(3, 1).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
}

#[test]
fn an_ended_baseline_keeps_what_the_primogenitor_left_as_the_singleton_control_group_harness() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(3).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_singleton_control_group_harness_that_was_created_is_given_a_directory_for_its_attempts() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(entries(outputs_directory.attempts(3)), 0);
}

#[test]
fn a_singleton_control_group_harness_that_was_not_created_is_given_no_directory_for_attempts() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert!(!outputs_directory.attempts(3).exists());
}

#[test]
fn the_source_code_of_a_singleton_control_group_harness_that_was_not_created_is_kept() {
    let (_directory, outputs_directory) = ended(CreationOutcome::TimedOut);
    assert!(outputs_directory.harness(3).join("new.txt").exists());
}

#[test]
fn an_ended_baseline_keeps_the_progenitor_the_child_and_the_outcome_of_its_experiment() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        names_in(outputs_directory.baseline(3)),
        ["child", "outcome", "progenitor"]
    );
}

#[test]
fn an_ended_baseline_records_the_progenitor_whose_source_code_was_changed() {
    assert_eq!(recorded(CreationOutcome::Created, "progenitor"), "1\n");
}

#[test]
fn an_ended_baseline_records_the_progenitor_s_child_of_its_experiment() {
    assert_eq!(recorded(CreationOutcome::Created, "child"), "2\n");
}

#[test]
fn an_ended_baseline_records_how_the_creation_ended() {
    assert_eq!(
        recorded(CreationOutcome::BuildFailed, "outcome"),
        "build_failed\n"
    );
}

#[test]
fn an_ended_baseline_is_no_longer_running() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert!(!outputs_directory.running_baseline(3).exists());
}

#[test]
fn a_singleton_control_group_harness_is_nobody_s_child() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(outputs_directory.progenitor_of(3), None);
}

#[test]
fn a_singleton_control_group_harness_is_not_one_of_the_children_of_the_progenitor() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(outputs_directory.children_of(1), Vec::<u64>::new());
}

#[test]
fn discarding_a_baseline_that_is_running_leaves_nothing_running() {
    let (_directory, outputs_directory) = started();
    outputs_directory.discard_baseline(3).unwrap();
    assert!(!outputs_directory.running_baseline(3).exists());
}

#[test]
fn a_discarded_baseline_can_be_started_again() {
    let (_directory, outputs_directory) = started();
    outputs_directory.discard_baseline(3).unwrap();
    assert!(outputs_directory.start_baseline(3, 1).is_ok());
}

#[test]
fn discarding_a_baseline_that_ended_removes_the_singleton_control_group_harness() {
    let (_directory, outputs_directory) = discarded_after_it_ended();
    assert!(!outputs_directory.harness(3).exists());
}

#[test]
fn discarding_a_baseline_that_ended_removes_what_was_kept_of_it() {
    let (_directory, outputs_directory) = discarded_after_it_ended();
    assert!(!outputs_directory.baseline(3).exists());
}

#[test]
fn discarding_a_baseline_that_ended_removes_the_directory_of_the_attempts() {
    let (_directory, outputs_directory) = discarded_after_it_ended();
    assert!(!outputs_directory.attempts(3).exists());
}

#[test]
fn discarding_a_baseline_leaves_the_progenitor_s_child_of_its_experiment() {
    let (_directory, outputs_directory) = discarded_after_it_ended();
    assert!(outputs_directory.harness(1).exists());
}

#[test]
fn a_baseline_recorded_as_unchanged_records_the_word_unchanged() {
    assert_eq!(
        recorded(CreationOutcome::Unchanged, "outcome"),
        "unchanged\n"
    );
}

#[test]
fn a_baseline_recorded_as_unchanged_leaves_no_directory_for_attempts() {
    let (_directory, outputs_directory) = ended(CreationOutcome::Unchanged);
    assert!(!outputs_directory.attempts(3).exists());
}
