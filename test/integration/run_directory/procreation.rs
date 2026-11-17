use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use cladenc::genealogy::ProcreationOutcome;
use cladenc::run_directory::RunDirectory;
use tempfile::TempDir;

fn started() -> (TempDir, RunDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let run_directory = RunDirectory::create(&directory.path().join("run"), &primogenitor).unwrap();
    run_directory.start_procreation(1, 0).unwrap();
    (directory, run_directory)
}

fn ended(outcome: ProcreationOutcome) -> (TempDir, RunDirectory) {
    let (directory, run_directory) = started();
    fs::write(
        run_directory.running_procreation(1).join("child/new.txt"),
        "new",
    )
    .unwrap();
    run_directory.end_procreation(1, outcome).unwrap();
    (directory, run_directory)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

fn harness_script(harness: PathBuf) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_run() {
    let (_directory, run_directory) = started();
    assert_eq!(
        harness_script(run_directory.running_procreation(1).join("progenitor")),
        harness_script(run_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_progenitor_to_modify() {
    let (_directory, run_directory) = started();
    assert_eq!(
        harness_script(run_directory.running_procreation(1).join("child")),
        harness_script(run_directory.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_its_two_copies_and_nothing_else() {
    let (_directory, run_directory) = started();
    assert_eq!(entries(run_directory.running_procreation(1)), 2);
}

#[test]
fn a_procreation_cannot_be_started_twice() {
    let (_directory, run_directory) = started();
    let refused = run_directory.start_procreation(1, 0).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn writing_into_the_progenitor_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, run_directory) = started();
    let copy = run_directory
        .running_procreation(1)
        .join("progenitor/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(run_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn writing_into_the_child_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, run_directory) = started();
    let copy = run_directory
        .running_procreation(1)
        .join("child/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(run_directory.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_the_files_of_the_child_as_its_harness() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(
        fs::read_to_string(run_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_an_empty_directory_for_its_attempts() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(entries(run_directory.attempts(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_keeps_neither_copy_in_its_directory() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Created);
    assert_eq!(entries(run_directory.procreation(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_leaves_nothing_running() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Created);
    assert!(!run_directory.running_procreation(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Failed);
    assert_eq!(
        fs::read_to_string(run_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_failed_leaves_no_directory_for_the_attempts_of_its_child() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Failed);
    assert!(!run_directory.attempts(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_neither_copy_in_its_directory() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Failed);
    assert_eq!(entries(run_directory.procreation(1)), 0);
}

#[test]
fn a_procreation_that_timed_out_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, run_directory) = ended(ProcreationOutcome::TimedOut);
    assert_eq!(
        fs::read_to_string(run_directory.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_timed_out_leaves_no_directory_for_the_attempts_of_its_child() {
    let (_directory, run_directory) = ended(ProcreationOutcome::TimedOut);
    assert!(!run_directory.attempts(1).exists());
}

#[test]
fn a_procreation_whose_child_did_not_build_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, run_directory) = ended(ProcreationOutcome::BuildFailed);
    assert!(run_directory.harness(1).join("harness.sh").is_file());
}

#[test]
fn a_procreation_whose_child_did_not_build_leaves_no_directory_for_its_attempts() {
    let (_directory, run_directory) = ended(ProcreationOutcome::BuildFailed);
    assert!(!run_directory.attempts(1).exists());
}

#[test]
fn a_procreation_that_failed_leaves_nothing_running() {
    let (_directory, run_directory) = ended(ProcreationOutcome::Failed);
    assert!(!run_directory.running_procreation(1).exists());
}
