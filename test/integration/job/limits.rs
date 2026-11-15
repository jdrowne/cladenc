use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use cladenc::genealogy::EvaluationOutcome;
use cladenc::job::{self, JobError};
use cladenc::workspace::Workspace;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use tempfile::TempDir;

type Evaluated = (TempDir, Workspace, Result<EvaluationOutcome, JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn written(pid_file: &Path) -> bool {
    fs::read_to_string(pid_file).is_ok_and(|pid| pid.ends_with('\n'))
}

fn within_ten_seconds(mut holds: impl FnMut() -> bool) -> bool {
    (0..1000).any(|_| {
        thread::sleep(Duration::from_millis(10));
        holds()
    })
}

fn gone(pid_file: &Path) -> bool {
    let pid = fs::read_to_string(pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    within_ten_seconds(|| kill(Pid::from_raw(pid), None).is_err())
}

fn timed_out() -> Evaluated {
    let (directory, workspace) = made();
    let evaluator = test_inputs::sample("integration", "spawn.sh");
    let stop = AtomicBool::new(false);
    let result = job::run_evaluation(&workspace, &evaluator, 0, "t1", 1, &stop);
    (directory, workspace, result)
}

fn stopped_while_running() -> Evaluated {
    let (directory, workspace) = made();
    let evaluator = test_inputs::sample("integration", "spawn.sh");
    let pid_file = workspace.running_evaluation(0, "t1").join("output/pid");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            within_ten_seconds(|| written(&pid_file));
            stop.store(true, Ordering::SeqCst);
        });
        job::run_evaluation(&workspace, &evaluator, 0, "t1", 60, &stop)
    });
    (directory, workspace, result)
}

#[test]
fn the_time_limit_kills_a_process_that_the_program_started() {
    let (_directory, workspace, _result) = timed_out();
    assert!(gone(&workspace.evaluation(0, "t1").join("output/pid")));
}

#[test]
fn a_job_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = stopped_while_running();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn a_job_whose_stop_flag_is_set_leaves_no_program_running() {
    let (_directory, workspace, _result) = stopped_while_running();
    assert!(gone(
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
        job::run_evaluation(&workspace, &evaluator, 0, "t1", 60, &stop),
        Err(JobError::Stopped)
    ));
}

#[test]
fn a_stopped_expansion_reports_that_it_was_stopped() {
    let (_directory, workspace) = made();
    let expander = test_inputs::sample("integration", "sleep.sh");
    let stop = AtomicBool::new(true);
    assert!(matches!(
        job::run_expansion(&workspace, &expander, 1, 0, 60, &stop),
        Err(JobError::Stopped)
    ));
}
