use std::path::Path;

use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Settings;

const IMAGE_BUILT: &str = "timeout -s KILL 35 docker build --tag debian:stable-slim --file \
                           OUTPUTS/image/Dockerfile OUTPUTS/image/context";

fn with_a_dockerfile(directory: &Path, settings: &mut Settings) {
    settings.dockerfile = Some(test_inputs::dockerfile(directory, ""));
    settings.docker_build_timeout = Some(30);
}

fn imaged_by(docker: Double) -> Evolved {
    given::started(docker, with_a_dockerfile)
}

fn not_built() -> Evolved {
    imaged_by(Double::default().answering(&["--file"], &[Answer::Exited(2)]))
}

#[test]
fn a_directed_evolution_given_a_dockerfile_first_builds_the_image_of_its_harnesses() {
    let commands = imaged_by(Double::default()).commands();
    assert_eq!(commands.first().map(String::as_str), Some(IMAGE_BUILT));
}

#[test]
fn an_image_that_is_built_is_told() {
    let said = imaged_by(Double::default()).said;
    assert!(said.contains("the image debian:stable-slim is built"));
}

#[test]
fn a_directed_evolution_given_no_dockerfile_builds_no_image_of_its_harnesses() {
    let evolved = given::started(Double::default(), given::unchanged);
    assert_eq!(evolved.count_of("--file"), 0);
}

#[test]
fn an_image_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        not_built().result,
        Err(DirectedEvolutionError::Image(ContainerError::Exit(2)))
    ));
}

#[test]
fn an_image_whose_build_fails_is_told_with_the_cause() {
    let said = not_built().said;
    assert!(said.contains("the build of the image exited with status 2"));
}

#[test]
fn no_environment_is_built_after_an_image_whose_build_fails() {
    assert_eq!(not_built().count_of("cladenc.task="), 0);
}

#[test]
fn a_directed_evolution_that_is_stopped_while_its_image_is_built_has_not_started() {
    let stopped = imaged_by(Double::default().answering(&["--file"], &[Answer::Stopped]));
    assert!(matches!(
        stopped.result,
        Err(DirectedEvolutionError::NotStarted)
    ));
}
