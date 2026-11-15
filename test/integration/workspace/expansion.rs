use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use cladenc::genealogy::ExpansionOutcome;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn started() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    workspace.start_expansion(1, 0).unwrap();
    (directory, workspace)
}

fn ended(outcome: ExpansionOutcome) -> (TempDir, Workspace) {
    let (directory, workspace) = started();
    fs::write(workspace.running_expansion(1).join("child/new.txt"), "new").unwrap();
    workspace.end_expansion(1, outcome).unwrap();
    (directory, workspace)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

fn harness_script(harness: PathBuf) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn a_started_expansion_holds_a_copy_of_its_parent_to_run() {
    let (_directory, workspace) = started();
    assert_eq!(
        harness_script(workspace.running_expansion(1).join("parent")),
        harness_script(workspace.harness(0))
    );
}

#[test]
fn a_started_expansion_holds_a_copy_of_its_parent_to_modify() {
    let (_directory, workspace) = started();
    assert_eq!(
        harness_script(workspace.running_expansion(1).join("child")),
        harness_script(workspace.harness(0))
    );
}

#[test]
fn a_started_expansion_holds_an_empty_output_directory() {
    let (_directory, workspace) = started();
    assert_eq!(entries(workspace.running_expansion(1).join("output")), 0);
}

#[test]
fn an_expansion_cannot_be_started_twice() {
    let (_directory, workspace) = started();
    let refused = workspace.start_expansion(1, 0).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn writing_into_the_parent_of_a_started_expansion_leaves_the_harness_unchanged() {
    let (_directory, workspace) = started();
    let copy = workspace.running_expansion(1).join("parent/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(workspace.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn writing_into_the_child_of_a_started_expansion_leaves_the_harness_unchanged() {
    let (_directory, workspace) = started();
    let copy = workspace.running_expansion(1).join("child/harness.sh");
    fs::write(copy, "changed").unwrap();
    assert_eq!(
        harness_script(workspace.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn an_expansion_that_created_its_child_leaves_the_files_of_the_child_as_its_harness() {
    let (_directory, workspace) = ended(ExpansionOutcome::Created);
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("new.txt")).unwrap(),
        "new"
    );
}

#[test]
fn an_expansion_that_created_its_child_leaves_an_empty_directory_for_its_evaluations() {
    let (_directory, workspace) = ended(ExpansionOutcome::Created);
    assert_eq!(entries(workspace.evaluations(1)), 0);
}

#[test]
fn an_expansion_that_created_its_child_keeps_only_its_output() {
    let (_directory, workspace) = ended(ExpansionOutcome::Created);
    let kept: Vec<_> = fs::read_dir(workspace.expansion(1))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(kept, ["output"]);
}

#[test]
fn an_expansion_that_created_its_child_leaves_nothing_running() {
    let (_directory, workspace) = ended(ExpansionOutcome::Created);
    assert!(!workspace.running_expansion(1).exists());
}

#[test]
fn an_expansion_that_failed_leaves_no_harness() {
    let (_directory, workspace) = ended(ExpansionOutcome::Failed);
    assert!(!workspace.harness(1).exists());
}

#[test]
fn an_expansion_that_failed_keeps_only_its_output() {
    let (_directory, workspace) = ended(ExpansionOutcome::Failed);
    let kept: Vec<_> = fs::read_dir(workspace.expansion(1))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(kept, ["output"]);
}

#[test]
fn an_expansion_that_timed_out_leaves_no_harness() {
    let (_directory, workspace) = ended(ExpansionOutcome::TimedOut);
    assert!(!workspace.harness(1).exists());
}

#[test]
fn an_expansion_that_failed_leaves_nothing_running() {
    let (_directory, workspace) = ended(ExpansionOutcome::Failed);
    assert!(!workspace.running_expansion(1).exists());
}
