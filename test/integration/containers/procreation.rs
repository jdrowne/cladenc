use std::fs;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::{AssessmentOutcome, ProcreationOutcome};
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use nix::unistd::Uid;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Procreated = (
    TempDir,
    Workspace,
    Result<ProcreationOutcome, ContainerError>,
);

fn made_at(run_directory: &str) -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join(run_directory), &progenitor).unwrap();
    (directory, workspace)
}

fn started_from(
    image: &str,
    run_command: &str,
    workspace: &Workspace,
    parent: u64,
    time_limit: u64,
) -> Result<ProcreationOutcome, ContainerError> {
    let launch = Launch {
        image,
        run_command,
        build_command: None,
        model: MODEL,
    };
    containers::run_procreation(
        workspace,
        &launch,
        1,
        parent,
        time_limit,
        &AtomicBool::new(false),
    )
}

fn procreated_by(run_command: &str, parent: u64, time_limit: u64) -> Procreated {
    let (directory, workspace) = made_at("run");
    let result = started_from(
        test_inputs::IMAGE,
        run_command,
        &workspace,
        parent,
        time_limit,
    );
    (directory, workspace, result)
}

fn procreated(run_command: &str) -> Procreated {
    procreated_by(run_command, 0, 30)
}

fn not_started_by_docker() -> Procreated {
    let (directory, workspace) = made_at("run");
    let result = started_from("Not An Image", "exit 0", &workspace, 0, 30);
    (directory, workspace, result)
}

fn left_in_the_child(run_command: &str, file: &str) -> String {
    let (_directory, workspace, _result) = procreated(run_command);
    fs::read_to_string(workspace.harness(1).join(file)).unwrap()
}

fn kept_of_the_procreation(run_command: &str, file: &str) -> String {
    let (_directory, workspace, _result) = procreated(run_command);
    fs::read_to_string(workspace.procreation(1).join(file)).unwrap()
}

fn left_by_a_harness_given_a_key(run_command: &str, file: &str) -> String {
    let (_directory, workspace) = made_at("run");
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: Model {
            key: Some(test_inputs::MODEL_KEY),
            ..MODEL
        },
    };
    let _ = containers::run_procreation(&workspace, &launch, 1, 0, 30, &AtomicBool::new(false));
    fs::read_to_string(workspace.harness(1).join(file)).unwrap()
}

fn heard_from_a_harness_that_says(said: &str) -> String {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let hearing = thread::spawn(move || {
        let mut heard = String::new();
        let _ = listener
            .accept()
            .and_then(|(mut harness, _)| harness.read_to_string(&mut heard));
        heard
    });
    let _ = procreated(&format!("echo {said} | nc host.docker.internal {port}"));
    drop(TcpStream::connect(("127.0.0.1", port)));
    hearing.join().unwrap()
}

fn harness_script(harness: &Path) -> String {
    fs::read_to_string(harness.join("harness.sh")).unwrap()
}

#[test]
fn the_working_directory_of_a_harness_is_the_copy_that_becomes_its_child() {
    assert_eq!(left_in_the_child("pwd > where", "where"), "/work\n");
}

#[test]
fn a_harness_is_told_where_its_source_code_is() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/harness\n"
    );
}

#[test]
fn the_source_code_a_harness_is_told_of_is_that_of_its_parent() {
    assert_eq!(
        left_in_the_child("cp \"$CLADENC_HARNESS/harness.sh\" copied", "copied"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_is_given_the_instruction_to_improve_its_source_code() {
    assert_eq!(
        left_in_the_child("printf %s \"$CLADENC_INSTRUCTION\" > given", "given"),
        containers::INSTRUCTION
    );
}

#[test]
fn a_harness_is_told_the_model_and_the_address_it_is_served_at() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_MODEL $CLADENC_MODEL_URL\" > told", "told"),
        format!("{} {}\n", test_inputs::MODEL, test_inputs::MODEL_URL)
    );
}

#[test]
fn a_harness_is_given_the_key_the_model_is_called_with() {
    assert_eq!(
        left_by_a_harness_given_a_key("printf %s \"$CLADENC_MODEL_KEY\" > given", "given"),
        test_inputs::MODEL_KEY
    );
}

#[test]
fn a_harness_of_a_search_with_no_key_is_given_none() {
    assert_eq!(
        left_in_the_child("echo \"${CLADENC_MODEL_KEY-none}\" > given", "given"),
        "none\n"
    );
}

#[test]
fn a_harness_reaches_the_machine_cladenc_runs_on_as_host_docker_internal() {
    assert_eq!(heard_from_a_harness_that_says("reached"), "reached\n");
}

#[test]
fn a_harness_can_read_the_ended_assessments_of_its_parent_and_none_still_running() {
    let (directory, workspace) = made_at("run");
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    workspace.start_assessment(0, "benchmark/t1").unwrap();
    workspace
        .end_assessment(0, "benchmark/t1", AssessmentOutcome::Failure)
        .unwrap();
    workspace.start_assessment(0, "benchmark/t2").unwrap();
    let _ = started_from(
        test_inputs::IMAGE,
        "ls /assessments /assessments/benchmark > evidence",
        &workspace,
        0,
        30,
    );
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("evidence")).unwrap(),
        "/assessments:\nbenchmark\n\n/assessments/benchmark:\nt1\n"
    );
}

#[test]
fn a_harness_cannot_write_into_the_assessments_of_its_parent() {
    let (_directory, workspace, _result) = procreated("touch /assessments/written");
    assert!(!workspace.assessments(0).join("written").exists());
}

#[test]
fn a_harness_runs_as_the_user_cladenc_runs_as() {
    assert_eq!(
        left_in_the_child("id -u > user", "user"),
        format!("{}\n", Uid::effective())
    );
}

#[test]
fn the_standard_input_of_a_harness_is_empty() {
    assert_eq!(left_in_the_child("cat > input", "input"), "");
}

#[test]
fn the_standard_output_of_a_harness_is_kept_with_its_procreation() {
    assert_eq!(kept_of_the_procreation("echo out", "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_a_harness_is_kept_with_its_procreation() {
    assert_eq!(kept_of_the_procreation("echo err >&2", "stderr"), "err\n");
}

#[test]
fn a_harness_that_exits_with_0_creates_its_child() {
    let (_directory, _workspace, result) = procreated("exit 0");
    assert!(matches!(result, Ok(ProcreationOutcome::Created)));
}

#[test]
fn a_procreation_with_no_build_command_leaves_no_build_of_its_child() {
    let (_directory, workspace, _result) = procreated("exit 0");
    assert!(!workspace.build(1).exists());
}

#[test]
fn a_procreation_that_created_its_child_leaves_a_directory_for_the_assessments_of_the_child() {
    let (_directory, workspace, _result) = procreated("exit 0");
    assert!(workspace.assessments(1).is_dir());
}

#[test]
fn a_harness_that_exits_with_1_fails_its_child() {
    let (_directory, _workspace, result) = procreated("exit 1");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_harness_that_exits_with_another_status_fails_its_child_too() {
    let (_directory, _workspace, result) = procreated("exit 2");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_harness_that_a_signal_ends_fails_its_child() {
    let (_directory, _workspace, result) = procreated("sh -c 'kill -TERM $$'; exit $?");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_procreation_that_failed_keeps_what_its_harness_left_beside_the_harnesses() {
    let (_directory, workspace, _result) = procreated("touch tried; exit 1");
    assert!(workspace.harness(1).join("tried").is_file());
}

#[test]
fn a_harness_that_runs_past_the_time_limit_times_out() {
    let (_directory, _workspace, result) = procreated_by("sleep 300", 0, 1);
    assert!(matches!(result, Ok(ProcreationOutcome::TimedOut)));
}

#[test]
fn the_time_limit_removes_the_container_of_a_harness() {
    let (_directory, workspace, _result) = procreated_by("hostname > id; sleep 300", 0, 5);
    assert!(test_inputs::container_is_gone(
        &workspace.harness(1).join("id")
    ));
}

#[test]
fn a_container_that_docker_does_not_start_is_an_error() {
    let (_directory, _workspace, result) = not_started_by_docker();
    assert!(matches!(result, Err(ContainerError::Docker(_))));
}

#[test]
fn a_container_that_ends_with_status_125_is_taken_as_one_docker_did_not_start() {
    let (_directory, _workspace, result) = procreated("exit 125");
    assert!(matches!(result, Err(ContainerError::Docker(_))));
}

#[test]
fn the_error_names_the_file_that_holds_what_docker_said() {
    let (_directory, workspace, result) = not_started_by_docker();
    assert!(matches!(
        result,
        Err(ContainerError::Docker(said)) if said == workspace.running_procreation(1).join("stderr")
    ));
}

#[test]
fn what_docker_said_is_kept_under_running() {
    let (_directory, workspace, _result) = not_started_by_docker();
    let said = fs::read_to_string(workspace.running_procreation(1).join("stderr")).unwrap();
    assert!(said.contains("invalid reference format"));
}

#[test]
fn a_procreation_of_a_harness_that_is_not_there_is_an_error() {
    let (_directory, _workspace, result) = procreated_by("exit 0", 9, 30);
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn a_harness_that_writes_into_the_source_code_it_runs_from_leaves_its_parent_unchanged() {
    let (_directory, workspace, _result) =
        procreated("echo changed > \"$CLADENC_HARNESS/harness.sh\"");
    assert_eq!(
        harness_script(&workspace.harness(0)),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_child_that_holds_a_named_pipe_is_failed() {
    let (_directory, _workspace, result) = procreated("mkfifo pipe");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_procreation_whose_directory_cannot_be_put_in_place_is_an_error() {
    let (_directory, workspace) = made_at("run");
    fs::create_dir_all(workspace.procreation(1).join("in the way")).unwrap();
    let result = started_from(test_inputs::IMAGE, "exit 0", &workspace, 0, 30);
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn a_run_directory_whose_path_holds_a_comma_a_quote_and_a_space_can_be_mounted() {
    let (_directory, workspace) = made_at("run,a\"b c");
    let result = started_from(test_inputs::IMAGE, "touch made", &workspace, 0, 30);
    assert!(matches!(result, Ok(ProcreationOutcome::Created)));
}
