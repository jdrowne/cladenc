use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings, Started};
use tempfile::TempDir;

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

const END_A_PROCREATION_WITH_125: &str = "if [ -d /var/log/cladenc/attempts ]; then exit 125; fi";

fn stopped_at_its_first_attempt() -> (TempDir, Result<Found, DirectedEvolutionError>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "exit 125");
    let outputs_directory = directory.path().join("outputs");
    let found = directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    (directory, found)
}

fn stopped_by_its_flag() -> (TempDir, Result<Found, DirectedEvolutionError>, String) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "hostname; sleep 300");
    let outputs_directory = directory.path().join("outputs");
    let id_file = outputs_directory.join("running/attempt-0-benchmark+t1/stdout");
    let stop = AtomicBool::new(false);
    let mut written = Vec::new();
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut written)
    });
    (directory, found, String::from_utf8(written).unwrap())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap()
}

#[test]
fn a_container_that_ends_with_status_125_at_an_attempt_stops_the_directed_evolution() {
    let (_directory, found) = stopped_at_its_first_attempt();
    assert!(matches!(
        found,
        Err(DirectedEvolutionError::Action {
            error: ContainerError::Docker(_),
            ..
        })
    ));
}

fn evolved_by(run_command: &str) -> Result<Found, DirectedEvolutionError> {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), run_command);
    let outputs_directory = directory.path().join("outputs");
    directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

#[test]
fn a_container_that_ends_with_status_125_at_a_procreation_stops_the_directed_evolution() {
    assert!(matches!(
        evolved_by(END_A_PROCREATION_WITH_125),
        Err(DirectedEvolutionError::Action {
            error: ContainerError::Docker(_),
            ..
        })
    ));
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_0_or_125_does_not_stop_the_directed_evolution() {
    assert!(evolved_by("exit 2").is_ok());
}

#[test]
fn a_directed_evolution_an_attempt_stopped_names_the_harness_and_the_task_of_that_attempt() {
    let (_directory, found) = stopped_at_its_first_attempt();
    assert!(matches!(
        found,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { harness: 0, task },
            ..
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_directed_evolution_a_container_stopped_names_the_progenitor_and_the_child_of_that_procreation()
{
    assert!(matches!(
        evolved_by(END_A_PROCREATION_WITH_125),
        Err(DirectedEvolutionError::Action {
            action: Started::Procreation {
                child: 1,
                progenitor: 0
            },
            ..
        })
    ));
}

#[test]
fn a_stopped_directed_evolution_leaves_a_started_and_no_finished_record_for_the_action_that_ran() {
    let (directory, _found) = stopped_at_its_first_attempt();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
    );
}

#[test]
fn a_stopped_directed_evolution_leaves_no_directed_evolution_finished_record() {
    let (directory, _found) = stopped_at_its_first_attempt();
    assert!(!journal_of(&directory).contains("directed_evolution_finished"));
}

#[test]
fn setting_the_stop_flag_stops_the_directed_evolution() {
    let (_directory, found, _said) = stopped_by_its_flag();
    assert!(matches!(found, Err(DirectedEvolutionError::Asked)));
}

#[test]
fn a_directed_evolution_stopped_by_its_flag_leaves_no_container_behind() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert!(test_inputs::container_is_gone(
        &directory
            .path()
            .join("outputs/running/attempt-0-benchmark+t1/stdout")
    ));
}

#[test]
fn a_directed_evolution_stopped_by_its_flag_writes_no_record_for_the_action_that_was_running() {
    let (directory, _found, _said) = stopped_by_its_flag();
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
    );
}

#[test]
fn a_directed_evolution_stopped_by_its_flag_says_that_it_was_asked_to_stop_and_what_was_running() {
    let (_directory, _found, said) = stopped_by_its_flag();
    assert!(said.ends_with("cladenc: asked to stop, with 1 container running\n"));
}

#[test]
fn a_directed_evolution_a_container_stopped_says_what_docker_did_when_it_happens() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), END_A_PROCREATION_WITH_125);
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let _stopped = directed_evolution::start(&settings, &outputs_directory, &stop, &mut written);
    assert!(String::from_utf8(written).unwrap().contains(
        "cladenc: the procreation of harness 1 by harness 0 could not be carried out: Docker \
         did not start its container"
    ));
}
