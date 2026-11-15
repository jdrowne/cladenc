use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
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

const SAY_THE_CONTAINER_AND_STAY: &str = "hostname > id; sleep 300";

type Evaluated = (TempDir, Workspace, Result<EvaluationOutcome, JobError>);
type Procreated = (TempDir, Workspace, Result<ProcreationOutcome, JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, workspace)
}

fn launch(run_command: &str) -> Launch<'_> {
    Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: MODEL,
    }
}

fn stopped_once_written<Outcome>(
    written: &Path,
    run: impl FnOnce(&AtomicBool) -> Result<Outcome, JobError>,
) -> Result<Outcome, JobError> {
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(written));
            stop.store(true, Ordering::SeqCst);
        });
        run(&stop)
    })
}

fn timed_out() -> Evaluated {
    let (directory, workspace) = made();
    let launch = launch(SAY_THE_CONTAINER_AND_STAY);
    let stop = AtomicBool::new(false);
    let result = job::run_evaluation(&workspace, &launch, 0, "benchmark/t1", 3, &stop);
    (directory, workspace, result)
}

fn stopped_while_its_harness_runs() -> Evaluated {
    let (directory, workspace) = made();
    let launch = launch(SAY_THE_CONTAINER_AND_STAY);
    let id_file = workspace
        .running_evaluation(0, "benchmark/t1")
        .join("work/id");
    let result = stopped_once_written(&id_file, |stop| {
        job::run_evaluation(&workspace, &launch, 0, "benchmark/t1", 60, stop)
    });
    (directory, workspace, result)
}

fn stopped_while_its_check_runs() -> Evaluated {
    let (directory, workspace) = made();
    test_inputs::kept_check(
        &directory.path().join("run"),
        "t1",
        SAY_THE_CONTAINER_AND_STAY,
    );
    let launch = launch("true");
    let id_file = workspace
        .running_evaluation(0, "benchmark/t1")
        .join("work/id");
    let result = stopped_once_written(&id_file, |stop| {
        job::run_evaluation(&workspace, &launch, 0, "benchmark/t1", 60, stop)
    });
    (directory, workspace, result)
}

fn procreation_stopped_while_running() -> Procreated {
    let (directory, workspace) = made();
    let launch = launch(SAY_THE_CONTAINER_AND_STAY);
    let id_file = workspace.running_procreation(1).join("child/id");
    let result = stopped_once_written(&id_file, |stop| {
        job::run_procreation(&workspace, &launch, 1, 0, 60, stop)
    });
    (directory, workspace, result)
}

#[test]
fn the_time_limit_of_an_evaluation_leaves_no_container_behind() {
    let (_directory, workspace, _result) = timed_out();
    assert!(test_inputs::container_is_gone(
        &workspace.evaluation(0, "benchmark/t1").join("work/id")
    ));
}

#[test]
fn an_evaluation_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = stopped_while_its_harness_runs();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn an_evaluation_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, workspace, _result) = stopped_while_its_harness_runs();
    assert!(test_inputs::container_is_gone(
        &workspace
            .running_evaluation(0, "benchmark/t1")
            .join("work/id")
    ));
}

#[test]
fn an_evaluation_whose_stop_flag_is_set_leaves_its_directory_under_running() {
    let (_directory, workspace, _result) = stopped_while_its_harness_runs();
    assert!(
        workspace
            .running_evaluation(0, "benchmark/t1")
            .join("harness")
            .is_dir()
    );
}

#[test]
fn an_evaluation_stopped_while_its_check_runs_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = stopped_while_its_check_runs();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn an_evaluation_stopped_while_its_check_runs_leaves_no_container_behind() {
    let (_directory, workspace, _result) = stopped_while_its_check_runs();
    assert!(test_inputs::container_is_gone(
        &workspace
            .running_evaluation(0, "benchmark/t1")
            .join("work/id")
    ));
}

#[test]
fn an_evaluation_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, workspace) = made();
    let stop = AtomicBool::new(true);
    assert!(matches!(
        job::run_evaluation(
            &workspace,
            &launch("sleep 300"),
            0,
            "benchmark/t1",
            60,
            &stop
        ),
        Err(JobError::Stopped)
    ));
}

#[test]
fn a_procreation_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, workspace) = made();
    let stop = AtomicBool::new(true);
    assert!(matches!(
        job::run_procreation(&workspace, &launch("sleep 300"), 1, 0, 60, &stop),
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
