use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::genealogy::{EvaluationOutcome, ProcreationOutcome};
use cladenc::job::{self, JobError, Launch};
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Evaluated = (TempDir, Workspace, Result<EvaluationOutcome, JobError>);
type Procreated = (TempDir, Workspace, Result<ProcreationOutcome, JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn timed_out() -> Evaluated {
    let (directory, workspace) = made();
    let evaluator = test_inputs::sample("integration", "spawn.sh");
    let stop = AtomicBool::new(false);
    let result = job::run_evaluation(&workspace, &evaluator, &MODEL, 0, "t1", 1, &stop);
    (directory, workspace, result)
}

fn stopped_while_running() -> Evaluated {
    let (directory, workspace) = made();
    let evaluator = test_inputs::sample("integration", "spawn.sh");
    let pid_file = workspace.running_evaluation(0, "t1").join("output/pid");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&pid_file));
            stop.store(true, Ordering::SeqCst);
        });
        job::run_evaluation(&workspace, &evaluator, &MODEL, 0, "t1", 60, &stop)
    });
    (directory, workspace, result)
}

fn procreation_stopped_while_running() -> Procreated {
    let (directory, workspace) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "hostname > id; sleep 300",
        build_command: None,
        model: MODEL,
    };
    let id_file = workspace.running_procreation(1).join("child/id");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        job::run_procreation(&workspace, &launch, 1, 0, 60, &stop)
    });
    (directory, workspace, result)
}

#[test]
fn the_time_limit_kills_a_process_that_the_program_started() {
    let (_directory, workspace, _result) = timed_out();
    assert!(test_inputs::process_is_gone(
        &workspace.evaluation(0, "t1").join("output/pid")
    ));
}

#[test]
fn a_job_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = stopped_while_running();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn a_job_whose_stop_flag_is_set_leaves_no_program_running() {
    let (_directory, workspace, _result) = stopped_while_running();
    assert!(test_inputs::process_is_gone(
        &workspace.running_evaluation(0, "t1").join("output/pid")
    ));
}

#[test]
fn a_job_whose_stop_flag_is_set_leaves_its_directory_under_running() {
    let (_directory, workspace, _result) = stopped_while_running();
    assert!(
        workspace
            .running_evaluation(0, "t1")
            .join("harness")
            .is_dir()
    );
}

#[test]
fn a_job_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, workspace) = made();
    let evaluator = test_inputs::sample("integration", "sleep.sh");
    let stop = AtomicBool::new(true);
    assert!(matches!(
        job::run_evaluation(&workspace, &evaluator, &MODEL, 0, "t1", 60, &stop),
        Err(JobError::Stopped)
    ));
}

#[test]
fn a_procreation_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, workspace) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "sleep 300",
        build_command: None,
        model: MODEL,
    };
    let stop = AtomicBool::new(true);
    assert!(matches!(
        job::run_procreation(&workspace, &launch, 1, 0, 60, &stop),
        Err(JobError::Stopped)
    ));
}

#[test]
fn a_procreation_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = procreation_stopped_while_running();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn a_procreation_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, workspace, _result) = procreation_stopped_while_running();
    assert!(test_inputs::container_is_gone(
        &workspace.running_procreation(1).join("child/id")
    ));
}
