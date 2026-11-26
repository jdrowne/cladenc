use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::containers::{self, ContainerError, Host, Launch, Program};
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
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

fn built(build_command: &str) -> Built {
    let (directory, outputs_directory) = made();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: Some(build_command),
        model: MODEL,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let result = containers::build_primogenitor(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        &AtomicBool::new(false),
    );
    (directory, outputs_directory, result)
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
