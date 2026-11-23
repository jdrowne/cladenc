use std::io::ErrorKind;
use std::path::PathBuf;

use crate::file_system::tests::double::Memory;
use crate::genealogy::CreationOutcome;
use crate::outputs_directory::OutputsDirectory;

fn started() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    outputs_directory.start_procreation(1, 0).unwrap();
    (directory, outputs_directory)
}

fn ended(outcome: CreationOutcome) -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    directory
        .write(
            outputs_directory
                .running_procreation(1)
                .join("child/new.txt"),
            "new",
        )
        .unwrap();
    outputs_directory.end_procreation(1, 0, outcome).unwrap();
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
        .read_to_string(outputs_directory.procreation(1).join(file))
        .unwrap()
}

fn harness_script(directory: &Memory, harness: PathBuf) -> String {
    directory
        .read_to_string(harness.join("harness.sh"))
        .unwrap()
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_run() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        harness_script(
            &directory,
            outputs_directory.running_procreation(1).join("progenitor")
        ),
        harness_script(&directory, outputs_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_modify() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        harness_script(
            &directory,
            outputs_directory.running_procreation(1).join("child")
        ),
        harness_script(&directory, outputs_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_its_two_copies_and_nothing_else() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        entries(&directory, outputs_directory.running_procreation(1)),
        2
    );
}

#[test]
fn a_procreation_cannot_be_started_twice() {
    let (_directory, outputs_directory) = started();
    let refused = outputs_directory.start_procreation(1, 0).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn writing_into_the_progenitor_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (directory, outputs_directory) = started();
    let copy = outputs_directory
        .running_procreation(1)
        .join("progenitor/harness.sh");
    directory.write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(&directory, outputs_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn writing_into_the_child_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (directory, outputs_directory) = started();
    let copy = outputs_directory
        .running_procreation(1)
        .join("child/harness.sh");
    directory.write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(&directory, outputs_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_the_files_of_the_child_as_its_harness() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        directory
            .read_to_string(outputs_directory.harness(1).join("new.txt"))
            .unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_an_empty_directory_for_its_attempts() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(entries(&directory, outputs_directory.attempts(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_keeps_its_record_and_neither_copy_in_its_directory() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert_eq!(
        names_in(&directory, outputs_directory.procreation(1)),
        ["outcome", "progenitor"]
    );
}

#[test]
fn a_procreation_that_ended_records_the_id_of_the_progenitor_on_a_line() {
    assert_eq!(recorded(CreationOutcome::Created, "progenitor"), "0\n");
}

#[test]
fn a_procreation_that_created_its_child_records_the_word_created() {
    assert_eq!(recorded(CreationOutcome::Created, "outcome"), "created\n");
}

#[test]
fn a_procreation_that_failed_records_the_word_failed() {
    assert_eq!(recorded(CreationOutcome::Failed, "outcome"), "failed\n");
}

#[test]
fn a_procreation_whose_child_did_not_build_records_the_word_build_failed() {
    assert_eq!(
        recorded(CreationOutcome::BuildFailed, "outcome"),
        "build_failed\n"
    );
}

#[test]
fn a_procreation_that_timed_out_records_the_word_timed_out() {
    assert_eq!(
        recorded(CreationOutcome::TimedOut, "outcome"),
        "timed_out\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_nothing_running() {
    let (directory, outputs_directory) = ended(CreationOutcome::Created);
    assert!(!directory.exists(&outputs_directory.running_procreation(1)));
}

#[test]
fn a_procreation_that_failed_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert_eq!(
        directory
            .read_to_string(outputs_directory.harness(1).join("new.txt"))
            .unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_failed_leaves_no_directory_for_the_attempts_of_its_child() {
    let (directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert!(!directory.exists(&outputs_directory.attempts(1)));
}

#[test]
fn a_procreation_that_failed_keeps_its_record_and_neither_copy_in_its_directory() {
    let (directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert_eq!(
        names_in(&directory, outputs_directory.procreation(1)),
        ["outcome", "progenitor"]
    );
}

#[test]
fn a_procreation_that_timed_out_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (directory, outputs_directory) = ended(CreationOutcome::TimedOut);
    assert_eq!(
        directory
            .read_to_string(outputs_directory.harness(1).join("new.txt"))
            .unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_timed_out_leaves_no_directory_for_the_attempts_of_its_child() {
    let (directory, outputs_directory) = ended(CreationOutcome::TimedOut);
    assert!(!directory.exists(&outputs_directory.attempts(1)));
}

#[test]
fn a_procreation_whose_child_did_not_build_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (directory, outputs_directory) = ended(CreationOutcome::BuildFailed);
    assert!(directory.is_file(&outputs_directory.harness(1).join("harness.sh")));
}

#[test]
fn a_procreation_whose_child_did_not_build_leaves_no_directory_for_its_attempts() {
    let (directory, outputs_directory) = ended(CreationOutcome::BuildFailed);
    assert!(!directory.exists(&outputs_directory.attempts(1)));
}

#[test]
fn a_procreation_that_failed_leaves_nothing_running() {
    let (directory, outputs_directory) = ended(CreationOutcome::Failed);
    assert!(!directory.exists(&outputs_directory.running_procreation(1)));
}

#[test]
fn a_procreation_recorded_as_unchanged_records_the_word_unchanged() {
    assert_eq!(
        recorded(CreationOutcome::Unchanged, "outcome"),
        "unchanged\n"
    );
}

#[test]
fn a_procreation_recorded_as_unchanged_keeps_what_its_harness_left_beside_the_harnesses() {
    let (directory, outputs_directory) = ended(CreationOutcome::Unchanged);
    assert!(directory.is_file(&outputs_directory.harness(1).join("harness.sh")));
}

#[test]
fn a_procreation_recorded_as_unchanged_leaves_no_directory_for_the_attempts_of_its_child() {
    let (directory, outputs_directory) = ended(CreationOutcome::Unchanged);
    assert!(!directory.exists(&outputs_directory.attempts(1)));
}
