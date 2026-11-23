use std::sync::atomic::AtomicBool;

use tempfile::TempDir;

use crate::containers::build;
use crate::containers::tests::double::Double;
use crate::containers::{ContainerError, Host, Launch};
use crate::model::Model;
use crate::outputs_directory::OutputsDirectory;

type Built = (TempDir, OutputsDirectory, Result<(), ContainerError>);

fn built_with_no_build_command() -> Built {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let docker = Double::default();
    let host = Host {
        docker: &docker,
        outputs_directory: &outputs_directory,
    };
    let result = build::build_primogenitor(host, &launch, &AtomicBool::new(false));
    (directory, outputs_directory, result)
}

#[test]
fn with_no_build_command_the_primogenitor_is_taken_as_built() {
    let (_directory, _outputs_directory, result) = built_with_no_build_command();
    assert!(result.is_ok());
}

#[test]
fn with_no_build_command_nothing_is_kept_under_builds() {
    let (_directory, outputs_directory, _result) = built_with_no_build_command();
    assert!(!outputs_directory.build(0).exists());
}
