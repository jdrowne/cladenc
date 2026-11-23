use std::path::Path;

use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Settings;

const BUILD: [&str; 2] = ["run", "make"];

fn with_a_build_command(_: &Path, settings: &mut Settings) {
    settings.harness_build_command = Some("make".to_owned());
}

fn built_by(docker: Double) -> Evolved {
    given::started(docker, with_a_build_command)
}

fn not_built() -> Evolved {
    built_by(Double::default().answering(&BUILD, &[Answer::Exited(1)]))
}

#[test]
fn a_primogenitor_with_a_build_command_is_built_before_it_attempts_a_task() {
    let said = built_by(Double::default()).said;
    assert!(said.contains("the primogenitor is built"));
}

#[test]
fn a_primogenitor_that_is_built_attempts_every_task() {
    assert_eq!(built_by(Double::default()).count_of("--detach"), 2);
}

#[test]
fn a_primogenitor_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        not_built().result,
        Err(DirectedEvolutionError::Build(ContainerError::Exit(1)))
    ));
}

#[test]
fn a_primogenitor_whose_build_fails_is_told_with_the_cause() {
    let said = not_built().said;
    assert!(said.contains("the build of the primogenitor exited with status 1"));
}

#[test]
fn a_primogenitor_whose_build_fails_attempts_no_task() {
    assert_eq!(not_built().count_of("--detach"), 0);
}

#[test]
fn a_directed_evolution_that_is_stopped_while_the_primogenitor_is_built_was_asked_to_stop() {
    let stopped = built_by(Double::default().answering(&BUILD, &[Answer::Stopped]));
    assert!(matches!(stopped.result, Err(DirectedEvolutionError::Asked)));
}
