use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::containers::{self, ContainerError, Launch};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Built = (TempDir, OutputsDirectory, Result<(), ContainerError>);

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn built_from(image: &str, build_command: &str, time_limit: u64, stop: bool) -> Built {
    let (directory, outputs_directory) = made();
    let launch = Launch {
        image,
        run_command: "exit 0",
        build_command: Some(build_command),
        model: MODEL,
        procreation_timeout: time_limit,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let stop = AtomicBool::new(stop);
    let result = containers::build_primogenitor(&outputs_directory, &launch, &stop);
    (directory, outputs_directory, result)
}

fn built(build_command: &str) -> Built {
    built_from(test_inputs::IMAGE, build_command, 30, false)
}

fn left_in_the_primogenitor(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = built(build_command);
    fs::read_to_string(outputs_directory.harness(0).join(file)).unwrap()
}

#[test]
fn the_primogenitor_is_built_in_its_place_among_the_harnesses() {
    assert_eq!(
        left_in_the_primogenitor("cp harness.sh product", "product"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn the_working_directory_of_the_build_of_the_primogenitor_is_its_source_code() {
    assert_eq!(
        left_in_the_primogenitor("pwd > where", "where"),
        "/var/opt/cladenc/harness\n"
    );
}

#[test]
fn the_build_of_the_primogenitor_is_told_where_the_source_code_is() {
    assert_eq!(
        left_in_the_primogenitor("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/var/opt/cladenc/harness\n"
    );
}

#[test]
fn the_container_of_the_build_of_the_primogenitor_holds_nothing_under_var_lib_named_after_cladenc()
{
    assert_eq!(
        left_in_the_primogenitor(
            "ls -d /var/lib/cladenc > held 2> /dev/null; echo looked >> held",
            "held"
        ),
        "looked\n"
    );
}

#[test]
fn the_build_of_the_primogenitor_finds_nothing_at_the_top_of_the_filesystem_that_its_image_lacks() {
    assert_eq!(
        left_in_the_primogenitor("ls / > top", "top"),
        test_inputs::printed_in_a_container_of(test_inputs::IMAGE, "ls /")
    );
}

#[test]
fn the_standard_output_of_the_build_of_the_primogenitor_is_kept_under_builds() {
    let (_directory, outputs_directory, _result) = built("echo out");
    assert_eq!(
        fs::read_to_string(outputs_directory.build(0).join("stdout")).unwrap(),
        "out\n"
    );
}

#[test]
fn the_standard_error_of_the_build_of_the_primogenitor_is_kept_under_builds() {
    let (_directory, outputs_directory, _result) = built("echo err >&2");
    assert_eq!(
        fs::read_to_string(outputs_directory.build(0).join("stderr")).unwrap(),
        "err\n"
    );
}

#[test]
fn a_primogenitor_whose_build_exits_with_0_is_built() {
    let (_directory, _outputs_directory, result) = built("exit 0");
    assert!(result.is_ok());
}

#[test]
fn a_primogenitor_whose_build_exits_with_another_status_is_an_error_that_holds_the_status() {
    let (_directory, _outputs_directory, result) = built("exit 3");
    assert!(matches!(result, Err(ContainerError::Exit(3))));
}

#[test]
fn a_primogenitor_whose_build_runs_past_the_time_limit_is_an_error() {
    let (_directory, _outputs_directory, result) =
        built_from(test_inputs::IMAGE, "sleep 300", 1, false);
    assert!(matches!(result, Err(ContainerError::TimeLimit)));
}

#[test]
fn a_primogenitor_whose_build_leaves_a_named_pipe_is_an_error() {
    let (_directory, _outputs_directory, result) = built("mkfifo pipe");
    assert!(matches!(result, Err(ContainerError::OtherFiles)));
}

#[test]
fn a_build_of_the_primogenitor_that_docker_does_not_start_is_an_error() {
    let (_directory, outputs_directory, result) = built_from("Not An Image", "exit 0", 30, false);
    assert!(matches!(
        result,
        Err(ContainerError::Docker(said)) if said == outputs_directory.build(0).join("stderr")
    ));
}

#[test]
fn a_build_of_the_primogenitor_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _outputs_directory, result) =
        built_from(test_inputs::IMAGE, "sleep 300", 60, true);
    assert!(matches!(result, Err(ContainerError::Stopped)));
}
