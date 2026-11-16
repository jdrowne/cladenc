use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::journal::{self, Settings, Started};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        budget: 6,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 0,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        assessment_timeout: 60,
        seed: 7,
    }
}

fn stopped_at_its_first_assessment() -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit 125");
    let run_directory = directory.path().join("run");
    let found = search::search(
        &settings,
        &run_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    (directory, found)
}

fn stopped_by_its_flag() -> (TempDir, Result<Found, SearchError>, String) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "hostname > id; sleep 300");
    let run_directory = directory.path().join("run");
    let id_file = run_directory.join("running/assessment-0-benchmark+t1/work/id");
    let stop = AtomicBool::new(false);
    let mut written = Vec::new();
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        search::search(&settings, &run_directory, &stop, &mut written)
    });
    (directory, found, String::from_utf8(written).unwrap())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

#[test]
fn a_container_that_ends_with_status_125_at_an_assessment_stops_the_search() {
    let (_directory, found) = stopped_at_its_first_assessment();
    assert!(matches!(
        found,
        Err(SearchError::Action {
            error: ContainerError::Docker(_),
            ..
        })
    ));
}

fn after_one_procreation_by(run_command: &str) -> Result<Found, SearchError> {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), run_command);
    settings.initial_procreations = 1;
    let run_directory = directory.path().join("run");
    search::search(
        &settings,
        &run_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

#[test]
fn a_container_that_ends_with_status_125_stops_the_search() {
    assert!(matches!(
        after_one_procreation_by("exit 125"),
        Err(SearchError::Action {
            error: ContainerError::Docker(_),
            ..
        })
    ));
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_0_or_125_does_not_stop_the_search() {
    assert!(after_one_procreation_by("exit 2").is_ok());
}

#[test]
fn a_search_an_assessment_stopped_names_the_harness_and_the_task_of_that_assessment() {
    let (_directory, found) = stopped_at_its_first_assessment();
    assert!(matches!(
        found,
        Err(SearchError::Action {
            action: Started::Assessment { harness: 0, task },
            ..
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_search_a_container_stopped_names_the_parent_and_the_child_of_that_procreation() {
    assert!(matches!(
        after_one_procreation_by("exit 125"),
        Err(SearchError::Action {
            action: Started::Procreation {
                child: 1,
                parent: 0
            },
            ..
        })
    ));
}

#[test]
fn a_stopped_search_leaves_a_started_record_and_no_finished_record_for_the_action_that_ran() {
    let (directory, _found) = stopped_at_its_first_assessment();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"assessment_started","harness":0,"task":"benchmark/t1"}"#
    );
}

#[test]
fn a_stopped_search_leaves_no_search_finished_record() {
    let (directory, _found) = stopped_at_its_first_assessment();
    assert!(!journal_of(&directory).contains("search_finished"));
}

#[test]
fn setting_the_stop_flag_stops_the_search() {
    let (_directory, found, _said) = stopped_by_its_flag();
    assert!(matches!(found, Err(SearchError::Asked)));
}

#[test]
fn a_search_stopped_by_its_flag_leaves_no_container_behind() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert!(test_inputs::container_is_gone(
        &directory
            .path()
            .join("run/running/assessment-0-benchmark+t1/work/id")
    ));
}

#[test]
fn a_search_stopped_by_its_flag_writes_no_record_for_the_action_that_was_running() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"assessment_started","harness":0,"task":"benchmark/t1"}"#
    );
}

#[test]
fn a_search_stopped_by_its_flag_says_that_it_was_asked_to_stop_and_what_was_running() {
    let (_directory, _found, said) = stopped_by_its_flag();
    assert!(said.ends_with("cladenc: asked to stop, with 1 container running\n"));
}

#[test]
fn a_search_a_container_stopped_says_what_docker_did_when_it_happens() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), "exit 125");
    settings.initial_procreations = 1;
    let run_directory = directory.path().join("run");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let _stopped = search::search(&settings, &run_directory, &stop, &mut written);
    assert!(String::from_utf8(written).unwrap().contains(
        "cladenc: the procreation of harness 1 by harness 0 could not be carried out: Docker \
         did not start its container"
    ));
}
