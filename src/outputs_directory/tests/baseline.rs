use std::io::ErrorKind;
use std::path::PathBuf;

use crate::file_system::tests::double::Memory;
use crate::genealogy::{CreationOutcome, Experiment};
use crate::outputs_directory::OutputsDirectory;

const EXPERIMENT: Experiment = Experiment {
    child: 2,
    progenitor: 1,
};

fn with_a_child_of_the_primogenitor() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    outputs_directory.start_diversification(1, 0).unwrap();
    directory
        .write(
            outputs_directory
                .running_diversification(1)
                .join("child/harness.sh"),
            "#!/bin/sh\necho child\n",
        )
        .unwrap();
    outputs_directory
        .end_diversification(1, 0, CreationOutcome::Created)
        .unwrap();
    (directory, outputs_directory)
}

fn started() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = with_a_child_of_the_primogenitor();
    outputs_directory.start_baseline(3, 1).unwrap();
    (directory, outputs_directory)
}

fn ended(outcome: CreationOutcome) -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    directory
        .write(
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

fn discarded_after_it_ended() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    outputs_directory.discard_baseline(3).unwrap();
    (directory, outputs_directory)
}

fn entries(files: &Memory, path: PathBuf) -> usize {
    files.names_in(path).len()
}

fn names_in(files: &Memory, path: PathBuf) -> Vec<String> {
    files.names_in(path)
}

fn recorded(outcome: CreationOutcome, file: &str) -> String {
    let (directory, outputs_directory) = ended(outcome);
    directory
        .read_to_string(outputs_directory.baseline(3).join(file))
        .unwrap()
}

fn harness_script(directory: &Memory, harness: PathBuf) -> String {
    directory
        .read_to_string(harness.join("harness.sh"))
        .unwrap()
}

#[test]
fn a_new_outputs_directory_holds_no_baseline() {
    let (directory, outputs_directory) = with_a_child_of_the_primogenitor();
    assert_eq!(
        entries(&directory, outputs_directory.path().join("baselines")),
        0
    );
}

#[test]
fn a_started_baseline_holds_a_copy_of_the_primogenitor_to_run() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        harness_script(
            &directory,
            outputs_directory.running_baseline(3).join("primogenitor")
        ),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_baseline_holds_a_clone_of_the_progenitor_to_diversify() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        harness_script(
            &directory,
            outputs_directory
                .running_baseline(3)
                .join("singleton-control-group-harness")
        ),
        "#!/bin/sh\necho child\n"
    );
}

#[test]
fn a_started_baseline_holds_its_copy_its_clone_and_nothing_else() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        entries(&directory, outputs_directory.running_baseline(3)),
        2
    );
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
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        directory
            .read_to_string(outputs_directory.harness(3).join("new.txt"))
            .unwrap(),
        "new"
    );
}

#[test]
fn a_singleton_control_group_harness_that_was_created_is_given_a_directory_for_its_trials() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(entries(&directory, outputs_directory.trials(3)), 0);
}

#[test]
fn a_singleton_control_group_harness_that_was_not_created_is_given_no_directory_for_trials() {
    let (directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert!(!directory.exists(outputs_directory.trials(3)));
}

#[test]
fn the_source_code_of_a_singleton_control_group_harness_that_was_not_created_is_kept() {
    let (directory, outputs_directory) = ended(CreationOutcome::TimedOut);
    assert!(directory.exists(outputs_directory.harness(3).join("new.txt")));
}

#[test]
fn an_ended_baseline_keeps_the_progenitor_the_child_and_the_outcome_of_its_experiment() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        names_in(&directory, outputs_directory.baseline(3)),
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
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert!(!directory.exists(outputs_directory.running_baseline(3)));
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
    let (directory, outputs_directory) = started();
    outputs_directory.discard_baseline(3).unwrap();
    assert!(!directory.exists(outputs_directory.running_baseline(3)));
}

#[test]
fn a_discarded_baseline_can_be_started_again() {
    let (_directory, outputs_directory) = started();
    outputs_directory.discard_baseline(3).unwrap();
    assert!(outputs_directory.start_baseline(3, 1).is_ok());
}

#[test]
fn discarding_a_baseline_that_ended_removes_the_singleton_control_group_harness() {
    let (directory, outputs_directory) = discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.harness(3)));
}

#[test]
fn discarding_a_baseline_that_ended_removes_what_was_kept_of_it() {
    let (directory, outputs_directory) = discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.baseline(3)));
}

#[test]
fn discarding_a_baseline_that_ended_removes_the_directory_of_the_trials() {
    let (directory, outputs_directory) = discarded_after_it_ended();
    assert!(!directory.exists(outputs_directory.trials(3)));
}

#[test]
fn discarding_a_baseline_leaves_the_progenitor_s_child_of_its_experiment() {
    let (directory, outputs_directory) = discarded_after_it_ended();
    assert!(directory.exists(outputs_directory.harness(1)));
}

#[test]
fn a_baseline_recorded_as_unchanged_records_the_word_unchanged() {
    assert_eq!(
        recorded(CreationOutcome::Unchanged, "outcome"),
        "unchanged\n"
    );
}

#[test]
fn a_baseline_recorded_as_unchanged_leaves_no_directory_for_trials() {
    let (directory, outputs_directory) = ended(CreationOutcome::Unchanged);
    assert!(!directory.exists(outputs_directory.trials(3)));
}
