use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::job::{self, JobError, Launch};
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Built = (TempDir, Workspace, Result<(), JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn built_from(image: &str, build_command: Option<&str>, time_limit: u64, stop: bool) -> Built {
    let (directory, workspace) = made();
    let launch = Launch {
        image,
        run_command: "exit 0",
        build_command,
        model: MODEL,
    };
    let stop = AtomicBool::new(stop);
    let result = job::build_progenitor(&workspace, &launch, time_limit, &stop);
    (directory, workspace, result)
}

fn built(build_command: &str) -> Built {
    built_from(test_inputs::IMAGE, Some(build_command), 30, false)
}

fn left_in_the_progenitor(build_command: &str, file: &str) -> String {
    let (_directory, workspace, _result) = built(build_command);
    fs::read_to_string(workspace.harness(0).join(file)).unwrap()
}

#[test]
fn the_progenitor_is_built_in_its_place_among_the_harnesses() {
    assert_eq!(
        left_in_the_progenitor("cp harness.sh product", "product"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn the_working_directory_of_the_build_of_the_progenitor_is_its_source_code() {
    assert_eq!(left_in_the_progenitor("pwd > where", "where"), "/harness\n");
}

#[test]
fn the_build_of_the_progenitor_is_told_where_the_source_code_is() {
    assert_eq!(
        left_in_the_progenitor("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/harness\n"
    );
}

#[test]
fn the_standard_output_of_the_build_of_the_progenitor_is_kept_under_builds() {
    let (_directory, workspace, _result) = built("echo out");
    assert_eq!(
        fs::read_to_string(workspace.build(0).join("stdout")).unwrap(),
        "out\n"
    );
}

#[test]
fn the_standard_error_of_the_build_of_the_progenitor_is_kept_under_builds() {
    let (_directory, workspace, _result) = built("echo err >&2");
    assert_eq!(
        fs::read_to_string(workspace.build(0).join("stderr")).unwrap(),
        "err\n"
    );
}

#[test]
fn a_progenitor_whose_build_exits_with_0_is_built() {
    let (_directory, _workspace, result) = built("exit 0");
    assert!(result.is_ok());
}

#[test]
fn a_progenitor_whose_build_exits_with_another_status_is_an_error_that_holds_the_status() {
    let (_directory, _workspace, result) = built("exit 3");
    assert!(matches!(result, Err(JobError::Exit(3))));
}

#[test]
fn a_progenitor_whose_build_runs_past_the_time_limit_is_an_error() {
    let (_directory, _workspace, result) =
        built_from(test_inputs::IMAGE, Some("sleep 300"), 1, false);
    assert!(matches!(result, Err(JobError::TimeLimit)));
}

#[test]
fn a_progenitor_whose_build_leaves_a_named_pipe_is_an_error() {
    let (_directory, _workspace, result) = built("mkfifo pipe");
    assert!(matches!(result, Err(JobError::OtherFiles)));
}

#[test]
fn a_build_of_the_progenitor_that_docker_does_not_start_is_an_error() {
    let (_directory, workspace, result) = built_from("Not An Image", Some("exit 0"), 30, false);
    assert!(matches!(
        result,
        Err(JobError::Docker(said)) if said == workspace.build(0).join("stderr")
    ));
}

#[test]
fn a_build_of_the_progenitor_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) =
        built_from(test_inputs::IMAGE, Some("sleep 300"), 60, true);
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn with_no_build_command_the_progenitor_is_taken_as_built() {
    let (_directory, _workspace, result) = built_from(test_inputs::IMAGE, None, 30, false);
    assert!(result.is_ok());
}

#[test]
fn with_no_build_command_nothing_is_kept_under_builds() {
    let (_directory, workspace, _result) = built_from(test_inputs::IMAGE, None, 30, false);
    assert!(!workspace.build(0).exists());
}
