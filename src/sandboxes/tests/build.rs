use std::sync::atomic::AtomicBool;

use crate::file_system::tests::double::Memory;
use crate::model::Model;
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::build;
use crate::sandboxes::tests::double::Double;
use crate::sandboxes::{Host, Launch, SandboxError};

type Built = (Memory, OutputsDirectory, Result<(), SandboxError>);

fn built_with_no_build_command() -> Built {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: "exit 0",
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
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
    let (directory, outputs_directory, _result) = built_with_no_build_command();
    assert!(!directory.exists(outputs_directory.build(0)));
}
