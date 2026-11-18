use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use cladenc::genealogy::ProcreationOutcome;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn started() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    outputs_directory.start_procreation(1, 0).unwrap();
    (directory, outputs_directory)
}

fn ended(outcome: ProcreationOutcome) -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = started();
    fs::write(
        outputs_directory
            .running_procreation(1)
            .join("child/new.txt"),
        "new",
    )
    .unwrap();
    outputs_directory.end_procreation(1, 0, outcome).unwrap();
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

fn recorded(outcome: ProcreationOutcome, file: &str) -> String {
    let (_directory, outputs_directory) = ended(outcome);
    fs::read_to_string(outputs_directory.procreation(1).join(file)).unwrap()
}

fn harness_script(harness: PathBuf) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_run() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        harness_script(outputs_directory.running_procreation(1).join("progenitor")),
        harness_script(outputs_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_modify() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        harness_script(outputs_directory.running_procreation(1).join("child")),
        harness_script(outputs_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_its_two_copies_and_nothing_else() {
    let (_directory, outputs_directory) = started();
    assert_eq!(entries(outputs_directory.running_procreation(1)), 2);
}

#[test]
fn a_procreation_cannot_be_started_twice() {
    let (_directory, outputs_directory) = started();
    let refused = outputs_directory.start_procreation(1, 0).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn writing_into_the_progenitor_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, outputs_directory) = started();
    let copy = outputs_directory
        .running_procreation(1)
        .join("progenitor/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(outputs_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn writing_into_the_child_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, outputs_directory) = started();
    let copy = outputs_directory
        .running_procreation(1)
        .join("child/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(outputs_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_the_files_of_the_child_as_its_harness() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_an_empty_directory_for_its_attempts() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(entries(outputs_directory.attempts(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_keeps_its_record_and_neither_copy_in_its_directory() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(
        names_in(outputs_directory.procreation(1)),
        ["outcome", "progenitor"]
    );
}

#[test]
fn a_procreation_that_ended_records_the_id_of_the_progenitor_on_a_line() {
    assert_eq!(recorded(ProcreationOutcome::Created, "progenitor"), "0\n");
}

#[test]
fn a_procreation_that_created_its_child_records_the_word_created() {
    assert_eq!(
        recorded(ProcreationOutcome::Created, "outcome"),
        "created\n"
    );
}

#[test]
fn a_procreation_that_failed_records_the_word_failed() {
    assert_eq!(recorded(ProcreationOutcome::Failed, "outcome"), "failed\n");
}

#[test]
fn a_procreation_whose_child_did_not_build_records_the_word_build_failed() {
    assert_eq!(
        recorded(ProcreationOutcome::BuildFailed, "outcome"),
        "build_failed\n"
    );
}

#[test]
fn a_procreation_that_timed_out_records_the_word_timed_out() {
    assert_eq!(
        recorded(ProcreationOutcome::TimedOut, "outcome"),
        "timed_out\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_nothing_running() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Created);
    assert!(!outputs_directory.running_procreation(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Failed);
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_failed_leaves_no_directory_for_the_attempts_of_its_child() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Failed);
    assert!(!outputs_directory.attempts(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_its_record_and_neither_copy_in_its_directory() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Failed);
    assert_eq!(
        names_in(outputs_directory.procreation(1)),
        ["outcome", "progenitor"]
    );
}

#[test]
fn a_procreation_that_timed_out_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::TimedOut);
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_timed_out_leaves_no_directory_for_the_attempts_of_its_child() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::TimedOut);
    assert!(!outputs_directory.attempts(1).exists());
}

#[test]
fn a_procreation_whose_child_did_not_build_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::BuildFailed);
    assert!(outputs_directory.harness(1).join("harness.sh").is_file());
}

#[test]
fn a_procreation_whose_child_did_not_build_leaves_no_directory_for_its_attempts() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::BuildFailed);
    assert!(!outputs_directory.attempts(1).exists());
}

#[test]
fn a_procreation_that_failed_leaves_nothing_running() {
    let (_directory, outputs_directory) = ended(ProcreationOutcome::Failed);
    assert!(!outputs_directory.running_procreation(1).exists());
}
