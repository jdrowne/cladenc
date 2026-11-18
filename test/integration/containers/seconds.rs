use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, Launch};
use cladenc::genealogy::{AttemptResult, ProcreationOutcome};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn launch<'a>(run_command: &'a str, build_command: Option<&'a str>, time_limit: u64) -> Launch<'a> {
    Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: time_limit,
        attempt_timeout: time_limit,
    }
}

fn made(grader: &str) -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", grader);
    (directory, outputs_directory)
}

fn attempted(run_command: &str, grader: &str, time_limit: u64) -> containers::Attempted {
    let (_directory, outputs_directory) = made(grader);
    containers::run_attempt(
        &outputs_directory,
        &launch(run_command, None, time_limit),
        0,
        "benchmark/t1",
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn procreated(run_command: &str, build_command: Option<&str>) -> containers::Procreated {
    let (_directory, outputs_directory) = made("exit 0");
    containers::run_procreation(
        &outputs_directory,
        &launch(run_command, build_command, 60),
        1,
        0,
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn the_seconds_of_an_attempt_are_those_its_harness_ran() {
    assert!(attempted("sleep 2", "exit 0", 60).seconds >= 2.0);
}

#[test]
fn the_seconds_of_an_attempt_leave_out_those_its_grader_ran() {
    assert!(attempted("exit 0", "sleep 15", 60).seconds < 15.0);
}

#[test]
fn the_seconds_of_an_attempt_that_ran_past_its_time_limit_are_no_fewer_than_the_limit() {
    assert!(attempted("sleep 300", "exit 0", 1).seconds >= 1.0);
}

#[test]
fn an_attempt_comes_back_with_its_result_beside_its_seconds() {
    assert_eq!(
        attempted("exit 0", "exit 0", 60).result,
        AttemptResult::Completed
    );
}

#[test]
fn the_seconds_of_an_attempt_are_given_to_the_millisecond() {
    let seconds = attempted("exit 0", "exit 0", 60).seconds;
    assert_eq!((seconds * 1000.0).round() / 1000.0, seconds);
}

#[test]
fn the_seconds_of_a_procreation_are_those_its_progenitor_ran() {
    assert!(procreated("sleep 2", None).seconds >= 2.0);
}

#[test]
fn the_seconds_of_a_procreation_leave_out_those_the_build_of_the_child_ran() {
    assert!(procreated("exit 0", Some("sleep 15")).seconds < 15.0);
}

#[test]
fn a_procreation_comes_back_with_its_outcome_beside_its_seconds() {
    assert_eq!(
        procreated("exit 0", None).outcome,
        ProcreationOutcome::Created
    );
}
