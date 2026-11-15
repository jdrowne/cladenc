use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::job::JobError;
use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn settings(directory: &Path, expander: &str, evaluator: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        tasks: vec!["t1".to_owned()],
        expander: test_inputs::sample("integration", expander),
        evaluator: test_inputs::sample("integration", evaluator),
        budget: 6,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_expansions: 0,
        workers: 1,
        expansion_timeout: 60,
        evaluation_timeout: 60,
        seed: 7,
    }
}

fn stopped_by_an_evaluator_that_exits_with_2() -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit-0.sh", "exit-2.sh");
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(false), None);
    (directory, found)
}

fn stopped_by_its_flag() -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit-0.sh", "spawn.sh");
    let run_directory = directory.path().join("run");
    let pid_file = run_directory.join("running/evaluation-0-t1/output/pid");
    let stop = AtomicBool::new(false);
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::pid_is_written(&pid_file));
            stop.store(true, Ordering::SeqCst);
        });
        search::search(&settings, &run_directory, &stop, None)
    });
    (directory, found)
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

#[test]
fn a_program_that_exits_with_a_status_other_than_0_or_1_stops_the_search() {
    let (_directory, found) = stopped_by_an_evaluator_that_exits_with_2();
    assert!(matches!(
        found,
        Err(SearchError::Stopped(Some(JobError::Exit(2))))
    ));
}

#[test]
fn an_expander_that_exits_with_a_status_other_than_0_or_1_stops_the_search() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), "exit-2.sh", "exit-0.sh");
    settings.initial_expansions = 1;
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(false), None);
    assert!(matches!(
        found,
        Err(SearchError::Stopped(Some(JobError::Exit(2))))
    ));
}

#[test]
fn a_stopped_search_leaves_a_started_record_and_no_finished_record_for_the_job_that_ran() {
    let (directory, _found) = stopped_by_an_evaluator_that_exits_with_2();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#
    );
}

#[test]
fn a_stopped_search_leaves_no_search_finished_record() {
    let (directory, _found) = stopped_by_an_evaluator_that_exits_with_2();
    assert!(!journal_of(&directory).contains("search_finished"));
}

#[test]
fn setting_the_stop_flag_stops_the_search() {
    let (_directory, found) = stopped_by_its_flag();
    assert!(matches!(found, Err(SearchError::Stopped(None))));
}

#[test]
fn a_search_stopped_by_its_flag_leaves_no_sample_program_running() {
    let (directory, _found) = stopped_by_its_flag();
    assert!(test_inputs::process_is_gone(
        &directory
            .path()
            .join("run/running/evaluation-0-t1/output/pid")
    ));
}

#[test]
fn a_search_stopped_by_its_flag_writes_no_record_for_the_job_that_was_running() {
    let (directory, _found) = stopped_by_its_flag();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#
    );
}
