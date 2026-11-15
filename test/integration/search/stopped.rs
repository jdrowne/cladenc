use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::job::JobError;
use cladenc::journal::{self, Settings, Started};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn settings(directory: &Path, run_command: &str, evaluator: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmark: test_inputs::benchmark(directory, &["t1"]),
        tasks: vec!["t1".to_owned()],
        docker_image: test_inputs::IMAGE.to_owned(),
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        evaluator: test_inputs::sample("integration", evaluator),
        budget: 6,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 0,
        workers: 1,
        procreation_timeout: 60,
        evaluation_timeout: 60,
        seed: 7,
    }
}

fn stopped_by_an_evaluator_that_exits_with_2() -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit 0", "exit-2.sh");
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(false), None);
    (directory, found)
}

fn stopped_by_its_flag() -> (TempDir, Result<Found, SearchError>, String) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit 0", "spawn.sh");
    let run_directory = directory.path().join("run");
    let pid_file = run_directory.join("running/evaluation-0-t1/output/pid");
    let stop = AtomicBool::new(false);
    let mut written = Vec::new();
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&pid_file));
            stop.store(true, Ordering::SeqCst);
        });
        search::search(&settings, &run_directory, &stop, Some(&mut written))
    });
    (directory, found, String::from_utf8(written).unwrap())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

#[test]
fn a_program_that_exits_with_a_status_other_than_0_or_1_stops_the_search() {
    let (_directory, found) = stopped_by_an_evaluator_that_exits_with_2();
    assert!(matches!(
        found,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

fn after_one_procreation_by(run_command: &str) -> Result<Found, SearchError> {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), run_command, "exit-0.sh");
    settings.initial_procreations = 1;
    let run_directory = directory.path().join("run");
    search::search(&settings, &run_directory, &AtomicBool::new(false), None)
}

#[test]
fn a_container_that_ends_with_status_125_stops_the_search() {
    assert!(matches!(
        after_one_procreation_by("exit 125"),
        Err(SearchError::Job {
            error: JobError::Docker(_),
            ..
        })
    ));
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_0_or_125_does_not_stop_the_search() {
    assert!(after_one_procreation_by("exit 2").is_ok());
}

#[test]
fn a_search_an_evaluator_stopped_names_the_harness_and_the_task_of_that_evaluation() {
    let (_directory, found) = stopped_by_an_evaluator_that_exits_with_2();
    assert!(matches!(
        found,
        Err(SearchError::Job {
            job: Started::Evaluation { harness: 0, task },
            ..
        }) if task == "t1"
    ));
}

#[test]
fn a_search_a_container_stopped_names_the_parent_and_the_child_of_that_procreation() {
    assert!(matches!(
        after_one_procreation_by("exit 125"),
        Err(SearchError::Job {
            job: Started::Procreation {
                child: 1,
                parent: 0
            },
            ..
        })
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
    let (_directory, found, _said) = stopped_by_its_flag();
    assert!(matches!(found, Err(SearchError::Asked)));
}

#[test]
fn a_search_stopped_by_its_flag_leaves_no_sample_program_running() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert!(test_inputs::process_is_gone(
        &directory
            .path()
            .join("run/running/evaluation-0-t1/output/pid")
    ));
}

#[test]
fn a_search_stopped_by_its_flag_writes_no_record_for_the_job_that_was_running() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#
    );
}

#[test]
fn a_search_stopped_by_its_flag_says_that_it_was_asked_to_stop_and_what_was_running() {
    let (_directory, _found, said) = stopped_by_its_flag();
    assert!(said.ends_with("cladenc: asked to stop, with 1 job running\n"));
}

#[test]
fn a_search_a_container_stopped_says_what_docker_did_when_it_happens() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), "exit 125", "exit-0.sh");
    settings.initial_procreations = 1;
    let run_directory = directory.path().join("run");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let _stopped = search::search(&settings, &run_directory, &stop, Some(&mut written));
    assert!(String::from_utf8(written).unwrap().contains(
        "cladenc: the procreation of harness 1 by harness 0 could not be carried out: Docker \
         did not start its container"
    ));
}
