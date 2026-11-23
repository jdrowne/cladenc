use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Started;

const HARNESS: [&str; 2] = ["exec", "run a harness"];

fn refused() -> Evolved {
    let refused = [Answer::Refused("docker is not there")];
    let docker = Double::default().answering(&["--detach"], &refused);
    given::started(docker, given::unchanged)
}

fn stopped() -> Evolved {
    let stop = Arc::new(AtomicBool::new(false));
    let asked = Arc::clone(&stop);
    let docker = Double::default()
        .answering(&HARNESS, &[Answer::UntilStopped])
        .doing(&HARNESS, move |_| asked.store(true, Ordering::SeqCst));
    given::started_until(docker, given::unchanged, &stop)
}

#[test]
fn an_attempt_that_cannot_be_carried_out_ends_the_directed_evolution_with_its_cause() {
    assert!(matches!(
        refused().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { harness: 0, task },
            error: ContainerError::Io(_),
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn an_attempt_that_cannot_be_carried_out_is_told_with_its_cause() {
    let said = refused().said;
    assert!(said.contains(
        "the attempt of harness 0 on task benchmark/t1 could not be carried out: docker is not \
         there"
    ));
}

#[test]
fn a_directed_evolution_tells_that_it_stops_when_an_attempt_cannot_be_carried_out() {
    let said = refused().said;
    assert!(said.contains("stopping the directed evolution, with"));
}

#[test]
fn nothing_is_started_after_an_attempt_that_cannot_be_carried_out() {
    assert_eq!(refused().count_of("--detach"), 1);
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_while_a_harness_runs_was_asked_to_stop() {
    assert!(matches!(
        stopped().result,
        Err(DirectedEvolutionError::Asked)
    ));
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_tells_so() {
    let said = stopped().said;
    assert!(said.contains("asked to stop, with"));
}

#[test]
fn nothing_is_started_after_a_directed_evolution_is_asked_to_stop() {
    assert_eq!(stopped().count_of("--detach"), 1);
}
