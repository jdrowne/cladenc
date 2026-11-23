use std::sync::atomic::AtomicBool;

use nix::unistd::{Gid, Uid};

use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{self, LAUNCH, TIMER};
use crate::containers::{self, ContainerError, Launch};
use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;

const BUILT: Launch<'static> = Launch {
    build_command: Some("make"),
    ..LAUNCH
};

const HARNESS: &str = ":/var/opt/cladenc/harness";

struct Built {
    _directory: Memory,
    outputs_directory: OutputsDirectory,
    docker: Double,
    result: Result<(), ContainerError>,
}

fn built_by(docker: Double) -> Built {
    let (directory, outputs_directory) = given::made();
    let docker = docker.on(&directory);
    let result = containers::build_primogenitor(
        given::host(&docker, &outputs_directory),
        &BUILT,
        &AtomicBool::new(false),
    );
    Built {
        _directory: directory,
        outputs_directory,
        docker,
        result,
    }
}

fn last_asked_of(built: &Built) -> Option<String> {
    built.docker.commands(&built.outputs_directory).pop()
}

#[test]
fn the_primogenitor_is_built_by_its_build_command_in_a_container_that_holds_its_source_code() {
    let built = built_by(Double::default());
    assert_eq!(
        built.docker.commands(&built.outputs_directory),
        [format!(
            "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
             --add-host host.docker.internal:host-gateway --user {}:{} \
             --workdir /var/opt/cladenc/harness \
             --volume OUTPUTS/harnesses/0:/var/opt/cladenc/harness --env CLADENC_HARNESS \
             debian:stable-slim sh -c {TIMER} make 35",
            Uid::effective(),
            Gid::effective()
        )]
    );
}

#[test]
fn the_build_of_the_primogenitor_is_told_where_its_source_code_is() {
    let built = built_by(Double::default());
    assert_eq!(
        built.docker.asked()[0].variables,
        [(
            "CLADENC_HARNESS".to_owned(),
            "/var/opt/cladenc/harness".to_owned()
        )]
    );
}

#[test]
fn what_the_build_of_the_primogenitor_prints_is_kept_with_its_builds() {
    let built = built_by(Double::default());
    assert_eq!(
        built.docker.asked()[0].kept,
        built.outputs_directory.build(0)
    );
}

#[test]
fn a_build_that_exits_with_zero_and_leaves_only_harness_files_has_built_the_primogenitor() {
    assert!(built_by(Double::default()).result.is_ok());
}

#[test]
fn a_build_that_leaves_something_that_is_not_a_harness_file_has_not_built_the_primogenitor() {
    let docker = given::leaving_a_socket(Double::default(), &["run"], HARNESS);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::OtherFiles)
    ));
}

#[test]
fn a_build_that_exits_with_another_status_fails_with_that_status() {
    let docker = Double::default().answering(&["run"], &[Answer::Exited(3)]);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::Exit(3))
    ));
}

#[test]
fn a_build_that_runs_past_the_time_limit_fails_with_the_time_limit() {
    let docker = Double::default().answering(&["run"], &[Answer::TimedOut]);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::TimeLimit)
    ));
}

#[test]
fn the_container_of_a_build_that_runs_past_the_time_limit_is_removed() {
    let built = built_by(Double::default().answering(&["run"], &[Answer::TimedOut]));
    assert_eq!(
        last_asked_of(&built).as_deref(),
        Some("docker rm --force NAME")
    );
}

#[test]
fn a_build_that_is_stopped_was_stopped() {
    let docker = Double::default().answering(&["run"], &[Answer::Stopped]);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn the_container_of_a_build_that_is_stopped_is_removed() {
    let built = built_by(Double::default().answering(&["run"], &[Answer::Stopped]));
    assert_eq!(
        last_asked_of(&built).as_deref(),
        Some("docker rm --force NAME")
    );
}

#[test]
fn a_build_whose_container_docker_does_not_start_names_the_file_that_holds_what_docker_said() {
    let built = built_by(Double::default().answering(&["run"], &[Answer::Exited(125)]));
    let said = built.outputs_directory.build(0).join("stderr");
    assert!(matches!(&built.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn a_build_that_a_signal_ends_fails_with_that_signal() {
    let docker = Double::default().answering(&["run"], &[Answer::Signal(9)]);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::Signal(9))
    ));
}

#[test]
fn a_build_that_docker_cannot_be_asked_for_is_not_carried_out() {
    let docker = Double::default().answering(&["run"], &[Answer::Refused("docker is not there")]);
    assert!(matches!(
        built_by(docker).result,
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn a_build_whose_directory_cannot_be_made_is_not_carried_out() {
    let (directory, outputs_directory) = given::made();
    directory
        .write(outputs_directory.path().join("builds"), "")
        .unwrap();
    let docker = Double::default();
    let docker = docker.on(&directory);
    let built = containers::build_primogenitor(
        given::host(&docker, &outputs_directory),
        &BUILT,
        &AtomicBool::new(false),
    );
    assert!(matches!(built, Err(ContainerError::Io(_))));
}
