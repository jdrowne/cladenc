use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::genealogy::{EvaluationOutcome, ExpansionOutcome};
use cladenc::job::{self, JobError};
use cladenc::workspace::Workspace;
use tempfile::TempDir;

type Expanded = (TempDir, Workspace, Result<ExpansionOutcome, JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn expanded_by(expander: PathBuf, parent: u64, time_limit: u64) -> Expanded {
    let (directory, workspace) = made();
    let stop = AtomicBool::new(false);
    let result = job::run_expansion(&workspace, &expander, 1, parent, time_limit, &stop);
    (directory, workspace, result)
}

fn expanded(sample: &str) -> Expanded {
    expanded_by(test_inputs::sample("integration", sample), 0, 30)
}

fn harness_script(workspace: &Workspace) -> String {
    fs::read_to_string(workspace.harness(0).join("harness.sh")).unwrap()
}

#[test]
fn the_expander_receives_the_parent_the_child_the_evidence_and_the_output_directory() {
    let (_directory, workspace, _result) = expanded("record.sh");
    let job = workspace.running_expansion(1);
    let arguments = [
        job.join("parent"),
        job.join("child"),
        workspace.evaluations(0),
        job.join("output"),
    ]
    .map(|argument| format!("{}\n", argument.display()))
    .concat();
    assert_eq!(
        fs::read_to_string(workspace.expansion(1).join("output/arguments")).unwrap(),
        arguments
    );
}

#[test]
fn an_expander_that_exits_with_0_creates_its_child() {
    let (_directory, _workspace, result) = expanded("exit-0.sh");
    assert!(matches!(result, Ok(ExpansionOutcome::Created)));
}

#[test]
fn an_expansion_that_created_its_child_leaves_the_child_as_a_harness() {
    let (_directory, workspace, _result) = expanded("exit-0.sh");
    assert!(workspace.harness(1).is_dir());
}

#[test]
fn an_expander_that_exits_with_1_fails_its_child() {
    let (_directory, _workspace, result) = expanded("exit-1.sh");
    assert!(matches!(result, Ok(ExpansionOutcome::Failed)));
}

#[test]
fn an_expander_that_runs_past_the_time_limit_times_out() {
    let (_directory, _workspace, result) =
        expanded_by(test_inputs::sample("integration", "sleep.sh"), 0, 1);
    assert!(matches!(result, Ok(ExpansionOutcome::TimedOut)));
}

#[test]
fn an_expander_that_exits_with_another_status_is_an_error() {
    let (_directory, _workspace, result) = expanded("exit-2.sh");
    assert!(matches!(result, Err(JobError::Exit(2))));
}

#[test]
fn an_expander_that_a_signal_ends_is_an_error() {
    let (_directory, _workspace, result) = expanded("killed.sh");
    assert!(matches!(result, Err(JobError::Signal(15))));
}

#[test]
fn an_expander_that_cannot_be_started_is_an_error() {
    let (_directory, _workspace, result) = expanded("missing.sh");
    assert!(matches!(result, Err(JobError::Io(_))));
}

#[test]
fn an_expansion_of_a_harness_that_is_not_there_is_an_error() {
    let (_directory, _workspace, result) =
        expanded_by(test_inputs::sample("integration", "exit-0.sh"), 9, 30);
    assert!(matches!(result, Err(JobError::Io(_))));
}

#[test]
fn after_an_error_the_directory_of_an_expansion_is_still_under_running() {
    let (_directory, workspace, _result) = expanded("exit-2.sh");
    assert!(workspace.running_expansion(1).join("stdout").is_file());
}

#[test]
fn an_expander_that_writes_into_its_parent_leaves_the_harness_unchanged() {
    let (_directory, workspace, _result) = expanded("write-into-first.sh");
    assert_eq!(harness_script(&workspace), "#!/bin/sh\necho harness\n");
}

#[test]
fn the_evidence_holds_the_ended_evaluations_of_the_parent_and_none_still_running() {
    let (_directory, workspace) = made();
    workspace.start_evaluation(0, "t1").unwrap();
    workspace
        .end_evaluation(0, "t1", EvaluationOutcome::Failure)
        .unwrap();
    workspace.start_evaluation(0, "t2").unwrap();
    let expander = test_inputs::sample("integration", "list-evidence.sh");
    let _ = job::run_expansion(&workspace, &expander, 1, 0, 30, &AtomicBool::new(false));
    assert_eq!(
        fs::read_to_string(workspace.expansion(1).join("output/evidence")).unwrap(),
        "t1\n"
    );
}

#[test]
fn a_child_that_holds_a_named_pipe_is_failed() {
    let (_directory, _workspace, result) = expanded("make-pipe.sh");
    assert!(matches!(result, Ok(ExpansionOutcome::Failed)));
}

#[test]
fn an_expander_that_removes_its_child_is_an_error() {
    let (_directory, _workspace, result) = expanded("remove-child.sh");
    assert!(matches!(result, Err(JobError::Io(_))));
}
