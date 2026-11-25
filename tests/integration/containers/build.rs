use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::containers::{self, ContainerError, Host, Launch, Program};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use nix::unistd::Uid;
use tempfile::TempDir;

const CHANGE_THE_COPY: &str = "touch changed";

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Procreated = (
    TempDir,
    OutputsDirectory,
    Result<containers::Creation, ContainerError>,
);

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
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
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_procreation(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        1,
        0,
        &stop,
    );
    (directory, outputs_directory, result)
}

fn procreated(run_command: &str, build_command: &str) -> Procreated {
    procreated_within(run_command, build_command, 30)
}

fn stopped_while_its_child_is_built() -> Procreated {
    let (directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: CHANGE_THE_COPY,
        build_command: Some("hostname > id; sleep 300"),
        model: MODEL,
        procreation_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let id_file = outputs_directory.running_procreation(1).join("child/id");
    let stop = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        containers::run_procreation(
            Host {
                docker: &Program,
                outputs_directory: &outputs_directory,
            },
            &launch,
            1,
            0,
            &stop,
        )
    });
    (directory, outputs_directory, result)
}

fn left_by_a_build_of_a_directed_evolution_with_a_key(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: CHANGE_THE_COPY,
        build_command: Some(build_command),
        model: Model {
            key: Some(test_inputs::MODEL_KEY),
            ..MODEL
        },
        procreation_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let _ = containers::run_procreation(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        1,
        0,
        &AtomicBool::new(false),
    );
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

fn left_in_the_child(build_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = procreated(CHANGE_THE_COPY, build_command);
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

#[test]
fn the_working_directory_of_the_build_of_a_child_is_the_source_code_of_the_child() {
    assert_eq!(
        left_in_the_child("pwd > where", "where"),
        "/var/opt/cladenc/harness\n"
    );
}

#[test]
fn the_build_of_a_child_is_told_where_the_source_code_is() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/var/opt/cladenc/harness\n"
    );
}

#[test]
fn the_container_of_the_build_of_a_child_holds_nothing_under_var_lib_named_after_cladenc() {
    assert_eq!(
        left_in_the_child(
            "ls -d /var/lib/cladenc > held 2> /dev/null; echo looked >> held",
            "held"
        ),
        "looked\n"
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
fn the_time_limit_removes_the_container_of_a_build() {
    let (_directory, outputs_directory, _result) =
        procreated_within(CHANGE_THE_COPY, "hostname > id; sleep 300", 8);
    assert!(test_inputs::container_is_gone(
        &outputs_directory.harness(1).join("id")
    ));
}

#[test]
fn a_procreation_stopped_while_its_child_is_built_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_child_is_built();
    assert!(test_inputs::container_is_gone(
        &outputs_directory.running_procreation(1).join("child/id")
    ));
}
