use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::ProcreationOutcome;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use nix::unistd::Uid;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Procreated = (
    TempDir,
    OutputsDirectory,
    Result<ProcreationOutcome, ContainerError>,
);

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn procreated_within(run_command: &str, build_command: &str, time_limit: u64) -> Procreated {
    let (directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: Some(build_command),
        model: MODEL,
        procreation_timeout: time_limit,
        attempt_timeout: time_limit,
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_procreation(&outputs_directory, &launch, 1, 0, &stop);
    (directory, outputs_directory, result)
}

fn procreated(run_command: &str, build_command: &str) -> Procreated {
    procreated_within(run_command, build_command, 30)
}

fn stopped_while_its_child_is_built() -> Procreated {
    let (directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some("hostname > id; sleep 300"),
        model: MODEL,
        procreation_timeout: 60,
        attempt_timeout: 60,
    };
    let id_file = outputs_directory.running_procreation(1).join("child/id");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        containers::run_procreation(&outputs_directory, &launch, 1, 0, &stop)
    });
    (directory, outputs_directory, result)
}

fn left_by_a_build_of_a_directed_evolution_with_a_key(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some(build_command),
        model: Model {
            key: Some(test_inputs::MODEL_KEY),
            ..MODEL
        },
        procreation_timeout: 30,
        attempt_timeout: 30,
    };
    let _ = containers::run_procreation(&outputs_directory, &launch, 1, 0, &AtomicBool::new(false));
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

fn left_in_the_child(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = procreated("exit 0", build_command);
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

fn kept_of_the_build(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = procreated("exit 0", build_command);
    fs::read_to_string(outputs_directory.build(1).join(file)).unwrap()
}

#[test]
fn the_working_directory_of_the_build_of_a_child_is_the_source_code_of_the_child() {
    assert_eq!(
        left_in_the_child("pwd > where", "where"),
        "/var/lib/cladenc/harness\n"
    );
}

#[test]
fn the_build_of_a_child_is_told_where_the_source_code_is() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/var/lib/cladenc/harness\n"
    );
}

#[test]
fn the_build_of_a_child_finds_nothing_at_the_top_of_the_filesystem_that_its_image_lacks() {
    assert_eq!(
        left_in_the_child("ls / > top", "top"),
        test_inputs::printed_in_a_container_of(test_inputs::IMAGE, "ls /")
    );
}

#[test]
fn a_build_is_told_nothing_of_the_model() {
    assert_eq!(
        left_by_a_build_of_a_directed_evolution_with_a_key(
            "echo \"${CLADENC_MODEL-none} ${CLADENC_MODEL_URL-none} ${CLADENC_MODEL_KEY-none}\" \
             > told",
            "told"
        ),
        "none none none\n"
    );
}

#[test]
fn the_build_of_a_child_works_on_what_its_progenitor_left() {
    let (_directory, outputs_directory, _result) =
        procreated("echo changed > source", "cp source product");
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(1).join("product")).unwrap(),
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
    let (_directory, outputs_directory, _result) = procreated("echo ran", "echo built");
    assert_eq!(
        fs::read_to_string(outputs_directory.procreation(1).join("stdout")).unwrap(),
        "ran\n"
    );
}

#[test]
fn a_child_whose_build_exits_with_0_is_created() {
    let (_directory, _outputs_directory, result) = procreated("exit 0", "exit 0");
    assert!(matches!(result, Ok(ProcreationOutcome::Created)));
}

#[test]
fn a_child_whose_build_exits_with_another_status_did_not_build() {
    let (_directory, _outputs_directory, result) = procreated("exit 0", "exit 1");
    assert!(matches!(result, Ok(ProcreationOutcome::BuildFailed)));
}

#[test]
fn a_child_that_did_not_build_is_kept_beside_the_harnesses() {
    let (_directory, outputs_directory, _result) = procreated("touch tried", "exit 1");
    assert!(outputs_directory.harness(1).join("tried").is_file());
}

#[test]
fn a_child_that_did_not_build_gets_no_directory_for_attempts() {
    let (_directory, outputs_directory, _result) = procreated("exit 0", "exit 1");
    assert!(!outputs_directory.attempts(1).exists());
}

#[test]
fn a_child_whose_build_leaves_a_named_pipe_did_not_build() {
    let (_directory, _outputs_directory, result) = procreated("exit 0", "mkfifo pipe");
    assert!(matches!(result, Ok(ProcreationOutcome::BuildFailed)));
}

#[test]
fn the_child_of_a_harness_that_failed_is_not_built() {
    let (_directory, outputs_directory, _result) = procreated("exit 1", "exit 0");
    assert!(!outputs_directory.build(1).exists());
}

#[test]
fn a_harness_that_failed_fails_its_child_whatever_the_build_would_have_done() {
    let (_directory, _outputs_directory, result) = procreated("exit 1", "exit 0");
    assert!(matches!(result, Ok(ProcreationOutcome::Failed)));
}

#[test]
fn a_build_that_runs_past_the_time_limit_of_its_procreation_times_the_procreation_out() {
    let (_directory, _outputs_directory, result) =
        procreated_within("exit 0", "touch started; sleep 300", 8);
    assert!(matches!(result, Ok(ProcreationOutcome::TimedOut)));
}

#[test]
fn the_time_limit_of_a_procreation_covers_its_harness_and_the_build_of_its_child_together() {
    let (_directory, _outputs_directory, result) = procreated_within("sleep 4", "sleep 4", 6);
    assert!(matches!(result, Ok(ProcreationOutcome::TimedOut)));
}

#[test]
fn the_time_limit_removes_the_container_of_a_build() {
    let (_directory, outputs_directory, _result) =
        procreated_within("exit 0", "hostname > id; sleep 300", 8);
    assert!(test_inputs::container_is_gone(
        &outputs_directory.harness(1).join("id")
    ));
}

#[test]
fn a_build_whose_container_ends_with_status_125_is_taken_as_one_docker_did_not_start() {
    let (_directory, _outputs_directory, result) = procreated("exit 0", "exit 125");
    assert!(matches!(result, Err(ContainerError::Docker(_))));
}

#[test]
fn the_error_of_a_build_names_the_file_under_builds_that_holds_what_docker_said() {
    let (_directory, outputs_directory, result) = procreated("exit 0", "exit 125");
    assert!(matches!(
        result,
        Err(ContainerError::Docker(said)) if said == outputs_directory.build(1).join("stderr")
    ));
}

#[test]
fn a_build_whose_output_cannot_be_kept_is_an_error() {
    let (directory, outputs_directory) = made();
    fs::write(directory.path().join("outputs/builds"), "").unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some("exit 0"),
        model: MODEL,
        procreation_timeout: 30,
        attempt_timeout: 30,
    };
    let stop = AtomicBool::new(false);
    assert!(matches!(
        containers::run_procreation(&outputs_directory, &launch, 1, 0, &stop),
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn a_procreation_stopped_while_its_child_is_built_reports_that_it_was_stopped() {
    let (_directory, _outputs_directory, result) = stopped_while_its_child_is_built();
    assert!(matches!(result, Err(ContainerError::Stopped)));
}

#[test]
fn a_procreation_stopped_while_its_child_is_built_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_child_is_built();
    assert!(test_inputs::container_is_gone(
        &outputs_directory.running_procreation(1).join("child/id")
    ));
}
