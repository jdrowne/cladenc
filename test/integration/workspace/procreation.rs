use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use cladenc::genealogy::ProcreationOutcome;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn started() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    workspace.start_procreation(1, 0).unwrap();
    (directory, workspace)
}

fn ended(outcome: ProcreationOutcome) -> (TempDir, Workspace) {
    let (directory, workspace) = started();
    fs::write(
        workspace.running_procreation(1).join("child/new.txt"),
        "new",
    )
    .unwrap();
    workspace.end_procreation(1, outcome).unwrap();
    (directory, workspace)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

fn harness_script(harness: PathBuf) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_parent_to_run() {
    let (_directory, workspace) = started();
    assert_eq!(
        harness_script(workspace.running_procreation(1).join("parent")),
        harness_script(workspace.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_a_copy_of_its_parent_to_modify() {
    let (_directory, workspace) = started();
    assert_eq!(
        harness_script(workspace.running_procreation(1).join("child")),
        harness_script(workspace.harness(0))
    );
}

#[test]
fn a_started_procreation_holds_its_two_copies_and_nothing_else() {
    let (_directory, workspace) = started();
    assert_eq!(entries(workspace.running_procreation(1)), 2);
}

#[test]
fn a_procreation_cannot_be_started_twice() {
    let (_directory, workspace) = started();
    let refused = workspace.start_procreation(1, 0).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn writing_into_the_parent_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, workspace) = started();
    let copy = workspace.running_procreation(1).join("parent/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(workspace.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn writing_into_the_child_of_a_started_procreation_leaves_the_harness_unchanged() {
    let (_directory, workspace) = started();
    let copy = workspace.running_procreation(1).join("child/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(workspace.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_the_files_of_the_child_as_its_harness() {
    let (_directory, workspace) = ended(ProcreationOutcome::Created);
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_created_its_child_leaves_an_empty_directory_for_its_evaluations() {
    let (_directory, workspace) = ended(ProcreationOutcome::Created);
    assert_eq!(entries(workspace.evaluations(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_keeps_neither_copy_in_its_directory() {
    let (_directory, workspace) = ended(ProcreationOutcome::Created);
    assert_eq!(entries(workspace.procreation(1)), 0);
}

#[test]
fn a_procreation_that_created_its_child_leaves_nothing_running() {
    let (_directory, workspace) = ended(ProcreationOutcome::Created);
    assert!(!workspace.running_procreation(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, workspace) = ended(ProcreationOutcome::Failed);
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_failed_leaves_no_directory_for_the_evaluations_of_its_child() {
    let (_directory, workspace) = ended(ProcreationOutcome::Failed);
    assert!(!workspace.evaluations(1).exists());
}

#[test]
fn a_procreation_that_failed_keeps_neither_copy_in_its_directory() {
    let (_directory, workspace) = ended(ProcreationOutcome::Failed);
    assert_eq!(entries(workspace.procreation(1)), 0);
}

#[test]
fn a_procreation_that_timed_out_keeps_the_files_of_the_child_beside_the_harnesses() {
    let (_directory, workspace) = ended(ProcreationOutcome::TimedOut);
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn a_procreation_that_timed_out_leaves_no_directory_for_the_evaluations_of_its_child() {
    let (_directory, workspace) = ended(ProcreationOutcome::TimedOut);
    assert!(!workspace.evaluations(1).exists());
}

#[test]
fn a_procreation_that_failed_leaves_nothing_running() {
    let (_directory, workspace) = ended(ProcreationOutcome::Failed);
    assert!(!workspace.running_procreation(1).exists());
}
