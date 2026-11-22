use std::path::Path;

use crate::cli::{Cli, Command};
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 13] = [
    ("--primogenitor", "/primogenitor"),
    ("--model", "a-model"),
    ("--model-url", "http://models.example/v1"),
    ("--benchmark", "/benchmark"),
    ("--harness-run-command", "run a harness"),
    ("--outputs-dir", "/outputs"),
    ("--max-procreations", "800"),
    ("--max-concurrent-containers", "8"),
    ("--procreation-timeout", "3600"),
    ("--max-grader-health-check-retries", "3"),
    ("--max-task-health-check-retries", "2"),
    ("--selection-tasks-percentage", "45"),
    ("--corroboration-tasks-percentage", "15"),
];

const IMAGE: &str = "--procreation-docker-image=image";
const DOCKERFILE: &str = "--procreation-dockerfile=/images/Dockerfile";
const TIME_LIMIT: &str = "--procreation-docker-image-build-timeout=600";

fn given(image: &[&str]) -> Result<Cli, clap::Error> {
    let options = OPTIONS
        .iter()
        .map(|(option, value)| format!("{option}={value}"));
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options)
            .chain(image.iter().map(|flag| (*flag).to_owned())),
    )
}

fn refusal(image: &[&str]) -> ErrorKind {
    given(image).unwrap_err().kind()
}

#[test]
fn an_image_is_accepted_with_no_dockerfile() {
    assert!(given(&[IMAGE]).is_ok());
}

#[test]
fn a_dockerfile_with_the_time_limit_of_its_build_is_accepted_in_place_of_an_image() {
    assert!(given(&[DOCKERFILE, TIME_LIMIT]).is_ok());
}

#[test]
fn an_image_and_a_dockerfile_together_are_an_error() {
    assert_eq!(
        refusal(&[IMAGE, DOCKERFILE, TIME_LIMIT]),
        ErrorKind::ArgumentConflict
    );
}

#[test]
fn neither_an_image_nor_a_dockerfile_is_an_error() {
    assert_eq!(refusal(&[]), ErrorKind::MissingRequiredArgument);
}

#[test]
fn a_dockerfile_without_the_time_limit_of_its_build_is_an_error() {
    assert_eq!(refusal(&[DOCKERFILE]), ErrorKind::MissingRequiredArgument);
}

#[test]
fn the_time_limit_of_a_build_beside_an_image_is_an_error() {
    assert_eq!(refusal(&[IMAGE, TIME_LIMIT]), ErrorKind::ArgumentConflict);
}

#[test]
fn the_time_limit_of_a_build_with_nothing_to_build_is_an_error() {
    assert_eq!(refusal(&[TIME_LIMIT]), ErrorKind::MissingRequiredArgument);
}

#[test]
fn a_time_limit_of_a_build_of_0_seconds_is_an_error() {
    assert_eq!(
        refusal(&[DOCKERFILE, "--procreation-docker-image-build-timeout=0"]),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_dockerfile_is_the_one_given() {
    assert!(matches!(
        given(&[DOCKERFILE, TIME_LIMIT]).unwrap().command,
        Command::Start(options)
            if options.procreation_dockerfile.as_deref() == Some(Path::new("/images/Dockerfile"))
    ));
}

#[test]
fn the_time_limit_of_the_build_is_the_one_given() {
    assert!(matches!(
        given(&[DOCKERFILE, TIME_LIMIT]).unwrap().command,
        Command::Start(options) if options.procreation_docker_image_build_timeout == Some(600)
    ));
}

#[test]
fn a_command_line_with_a_dockerfile_names_no_image() {
    assert!(matches!(
        given(&[DOCKERFILE, TIME_LIMIT]).unwrap().command,
        Command::Start(options) if options.procreation_docker_image.is_none()
    ));
}

#[test]
fn a_command_line_with_an_image_names_no_dockerfile() {
    assert!(matches!(
        given(&[IMAGE]).unwrap().command,
        Command::Start(options) if options.procreation_dockerfile.is_none()
    ));
}

#[test]
fn the_image_is_the_one_given() {
    assert!(matches!(
        given(&[IMAGE]).unwrap().command,
        Command::Start(options) if options.procreation_docker_image.as_deref() == Some("image")
    ));
}
