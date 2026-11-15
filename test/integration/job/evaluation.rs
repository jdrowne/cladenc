use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::genealogy::EvaluationOutcome;
use cladenc::job::{self, JobError};
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

type Evaluated = (TempDir, Workspace, Result<EvaluationOutcome, JobError>);

fn evaluated_with(
    evaluator: PathBuf,
    key: Option<&str>,
    harness: u64,
    time_limit: u64,
) -> Evaluated {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let model = Model {
        name: test_inputs::MODEL,
        url: test_inputs::MODEL_URL,
        key,
    };
    let stop = AtomicBool::new(false);
    let result = job::run_evaluation(
        &workspace, &evaluator, &model, harness, "t1", time_limit, &stop,
    );
    (directory, workspace, result)
}

fn evaluated_by(evaluator: PathBuf, harness: u64, time_limit: u64) -> Evaluated {
    evaluated_with(evaluator, None, harness, time_limit)
}

fn evaluated(sample: &str) -> Evaluated {
    evaluated_by(test_inputs::sample("integration", sample), 0, 30)
}

fn told_of_the_model(key: Option<&str>) -> Vec<String> {
    let evaluator = test_inputs::sample("integration", "record.sh");
    let (_directory, workspace, _result) = evaluated_with(evaluator, key, 0, 30);
    let environment =
        fs::read_to_string(workspace.evaluation(0, "t1").join("output/environment")).unwrap();
    let mut told: Vec<String> = environment
        .lines()
        .filter(|variable| variable.starts_with("CLADENC_MODEL"))
        .map(str::to_owned)
        .collect();
    told.sort();
    told
}

#[test]
fn the_evaluator_receives_the_harness_the_task_and_the_output_directory() {
    let (_directory, workspace, _result) = evaluated("record.sh");
    let job = workspace.running_evaluation(0, "t1");
    let arguments = format!(
        "{}\nt1\n{}\n",
        job.join("harness").display(),
        job.join("output").display()
    );
    assert_eq!(
        fs::read_to_string(workspace.evaluation(0, "t1").join("output/arguments")).unwrap(),
        arguments
    );
}

#[test]
fn the_evaluator_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    assert_eq!(
        told_of_the_model(Some(test_inputs::MODEL_KEY)),
        [
            format!("CLADENC_MODEL={}", test_inputs::MODEL),
            format!("CLADENC_MODEL_KEY={}", test_inputs::MODEL_KEY),
            format!("CLADENC_MODEL_URL={}", test_inputs::MODEL_URL),
        ]
    );
}

#[test]
fn the_evaluator_of_a_search_with_no_key_is_given_none() {
    assert_eq!(
        told_of_the_model(None),
        [
            format!("CLADENC_MODEL={}", test_inputs::MODEL),
            format!("CLADENC_MODEL_URL={}", test_inputs::MODEL_URL),
        ]
    );
}

#[test]
fn an_evaluator_that_exits_with_0_is_a_success() {
    let (_directory, _workspace, result) = evaluated("exit-0.sh");
    assert!(matches!(result, Ok(EvaluationOutcome::Success)));
}

#[test]
fn an_evaluation_that_ended_leaves_its_outcome_in_its_place() {
    let (_directory, workspace, _result) = evaluated("exit-0.sh");
    assert_eq!(
        fs::read_to_string(workspace.evaluation(0, "t1").join("outcome")).unwrap(),
        "success\n"
    );
}

#[test]
fn an_evaluator_that_exits_with_1_is_a_failure() {
    let (_directory, _workspace, result) = evaluated("exit-1.sh");
    assert!(matches!(result, Ok(EvaluationOutcome::Failure)));
}

#[test]
fn an_evaluator_that_runs_past_the_time_limit_times_out() {
    let (_directory, _workspace, result) =
        evaluated_by(test_inputs::sample("integration", "sleep.sh"), 0, 1);
    assert!(matches!(result, Ok(EvaluationOutcome::TimedOut)));
}

#[test]
fn an_evaluator_that_exits_with_another_status_is_an_error() {
    let (_directory, _workspace, result) = evaluated("exit-2.sh");
    assert!(matches!(result, Err(JobError::Exit(2))));
}

#[test]
fn an_evaluator_that_a_signal_ends_is_an_error() {
    let (_directory, _workspace, result) = evaluated("killed.sh");
    assert!(matches!(result, Err(JobError::Signal(15))));
}

#[test]
fn an_evaluator_that_cannot_be_started_is_an_error() {
    let (_directory, _workspace, result) = evaluated("missing.sh");
    assert!(matches!(result, Err(JobError::Io(_))));
}

#[test]
fn an_evaluation_of_a_harness_that_is_not_there_is_an_error() {
    let (_directory, _workspace, result) =
        evaluated_by(test_inputs::sample("integration", "exit-0.sh"), 9, 30);
    assert!(matches!(result, Err(JobError::Io(_))));
}

#[test]
fn after_an_error_the_directory_of_an_evaluation_is_still_under_running() {
    let (_directory, workspace, _result) = evaluated("exit-2.sh");
    assert!(
        workspace
            .running_evaluation(0, "t1")
            .join("stdout")
            .is_file()
    );
}

#[test]
fn an_evaluator_that_writes_into_its_harness_leaves_the_harness_unchanged() {
    let (_directory, workspace, _result) = evaluated("write-into-first.sh");
    assert_eq!(
        fs::read_to_string(workspace.harness(0).join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}
