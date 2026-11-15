use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::genealogy::ProcreationOutcome;
use cladenc::job::{self, JobError, Launch};
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use nix::unistd::Uid;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Procreated = (TempDir, Workspace, Result<ProcreationOutcome, JobError>);

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn procreated_within(run_command: &str, build_command: &str, time_limit: u64) -> Procreated {
    let (directory, workspace) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: Some(build_command),
        model: MODEL,
    };
    let stop = AtomicBool::new(false);
    let result = job::run_procreation(&workspace, &launch, 1, 0, time_limit, &stop);
    (directory, workspace, result)
}

fn procreated(run_command: &str, build_command: &str) -> Procreated {
    procreated_within(run_command, build_command, 30)
}

fn stopped_while_its_child_is_built() -> Procreated {
    let (directory, workspace) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some("hostname > id; sleep 300"),
        model: MODEL,
    };
    let id_file = workspace.running_procreation(1).join("child/id");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        job::run_procreation(&workspace, &launch, 1, 0, 60, &stop)
    });
    (directory, workspace, result)
}

fn left_by_a_build_of_a_search_with_a_key(build_command: &str, file: &str) -> String {
    let (_directory, workspace) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some(build_command),
        model: Model {
            key: Some(test_inputs::MODEL_KEY),
            ..MODEL
        },
    };
    let _ = job::run_procreation(&workspace, &launch, 1, 0, 30, &AtomicBool::new(false));
    fs::read_to_string(workspace.harness(1).join(file)).unwrap()
}

fn left_in_the_child(build_command: &str, file: &str) -> String {
    let (_directory, workspace, _result) = procreated("exit 0", build_command);
    fs::read_to_string(workspace.harness(1).join(file)).unwrap()
}

fn kept_of_the_build(build_command: &str, file: &str) -> String {
    let (_directory, workspace, _result) = procreated("exit 0", build_command);
    fs::read_to_string(workspace.build(1).join(file)).unwrap()
}

#[test]
fn the_working_directory_of_the_build_of_a_child_is_the_source_code_of_the_child() {
    assert_eq!(left_in_the_child("pwd > where", "where"), "/harness\n");
}

#[test]
fn the_build_of_a_child_is_told_where_the_source_code_is() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/harness\n"
    );
}

#[test]
fn a_build_is_told_nothing_of_the_model() {
    assert_eq!(
        left_by_a_build_of_a_search_with_a_key(
            "echo \"${CLADENC_MODEL-none} ${CLADENC_MODEL_URL-none} ${CLADENC_MODEL_KEY-none}\" \
             > told",
            "told"
        ),
        "none none none\n"
    );
}

#[test]
fn the_build_of_a_child_works_on_what_its_parent_left() {
    let (_directory, workspace, _result) = procreated("echo changed > source", "cp source product");
    assert_eq!(
        fs::read_to_string(workspace.harness(1).join("product")).unwrap(),
        "changed\n"
    );
}

#[test]
fn the_build_of_a_child_runs_as_the_user_cladenc_runs_as() {
    assert_eq!(
        left_in_the_child("id -u > user", "user"),
        format!("{}\n", Uid::effective())
    );
}

#[test]
fn the_standard_output_of_the_build_of_a_child_is_kept_under_builds() {
    assert_eq!(kept_of_the_build("echo out", "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_the_build_of_a_child_is_kept_under_builds() {
    assert_eq!(kept_of_the_build("echo err >&2", "stderr"), "err\n");
}

#[test]
fn the_standard_output_of_a_harness_is_kept_apart_from_that_of_the_build_of_its_child() {
    let (_directory, workspace, _result) = procreated("echo ran", "echo built");
    assert_eq!(
        fs::read_to_string(workspace.procreation(1).join("stdout")).unwrap(),
        "ran\n"
    );
}

#[test]
fn a_child_whose_build_exits_with_0_is_created() {
    let (_directory, _workspace, result) = procreated("exit 0", "exit 0");
    assert!(matches!(result, Ok(ProcreationOutcome::Created)));
}

#[test]
fn a_child_whose_build_exits_with_another_status_did_not_build() {
    let (_directory, _workspace, result) = procreated("exit 0", "exit 1");
    assert!(matches!(result, Ok(ProcreationOutcome::BuildFailed)));
}

#[test]
fn a_child_that_did_not_build_is_kept_beside_the_harnesses() {
    let (_directory, workspace, _result) = procreated("touch tried", "exit 1");
    assert!(workspace.harness(1).join("tried").is_file());
}

#[test]
fn a_child_that_did_not_build_gets_no_directory_for_evaluations() {
    let (_directory, workspace, _result) = procreated("exit 0", "exit 1");
    assert!(!workspace.evaluations(1).exists());
}

#[test]
fn a_child_whose_build_leaves_a_named_pipe_did_not_build() {
    let (_directory, _workspace, result) = procreated("exit 0", "mkfifo pipe");
    assert!(matches!(result, Ok(ProcreationOutcome::BuildFailed)));
}

#[test]
fn the_child_of_a_harness_that_failed_is_not_built() {
    let (_directory, workspace, _result) = procreated("exit 1", "exit 0");
    assert!(!workspace.build(1).exists());
}

#[test]
fn a_harness_that_failed_fails_its_child_whatever_the_build_would_have_done() {
    let (_directory, _workspace, result) = procreated("exit 1", "exit 0");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_build_that_runs_past_the_time_limit_of_its_procreation_times_the_procreation_out() {
    let (_directory, _workspace, result) =
        procreated_within("exit 0", "touch started; sleep 300", 8);
    assert!(matches!(result, Ok(ProcreationOutcome::TimedOut)));
}

#[test]
fn the_time_limit_of_a_procreation_covers_its_harness_and_the_build_of_its_child_together() {
    let (_directory, _workspace, result) = procreated_within("sleep 4", "sleep 4", 6);
    assert!(matches!(result, Ok(ProcreationOutcome::TimedOut)));
}

#[test]
fn the_time_limit_removes_the_container_of_a_build() {
    let (_directory, workspace, _result) =
        procreated_within("exit 0", "hostname > id; sleep 300", 8);
    assert!(test_inputs::container_is_gone(
        &workspace.harness(1).join("id")
    ));
}

#[test]
fn a_build_whose_container_ends_with_status_125_is_taken_as_one_docker_did_not_start() {
    let (_directory, _workspace, result) = procreated("exit 0", "exit 125");
    assert!(matches!(result, Err(JobError::Docker(_))));
}

#[test]
fn the_error_of_a_build_names_the_file_under_builds_that_holds_what_docker_said() {
    let (_directory, workspace, result) = procreated("exit 0", "exit 125");
    assert!(matches!(
        result,
        Err(JobError::Docker(said)) if said == workspace.build(1).join("stderr")
    ));
}

#[test]
fn a_build_whose_output_cannot_be_kept_is_an_error() {
    let (directory, workspace) = made();
    fs::write(directory.path().join("run/builds"), "").unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some("exit 0"),
        model: MODEL,
    };
    let stop = AtomicBool::new(false);
    assert!(matches!(
        job::run_procreation(&workspace, &launch, 1, 0, 30, &stop),
        Err(JobError::Io(_))
    ));
}

#[test]
fn a_procreation_stopped_while_its_child_is_built_reports_that_it_was_stopped() {
    let (_directory, _workspace, result) = stopped_while_its_child_is_built();
    assert!(matches!(result, Err(JobError::Stopped)));
}

#[test]
fn a_procreation_stopped_while_its_child_is_built_leaves_no_container_behind() {
    let (_directory, workspace, _result) = stopped_while_its_child_is_built();
    assert!(test_inputs::container_is_gone(
        &workspace.running_procreation(1).join("child/id")
    ));
}
