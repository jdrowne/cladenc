use std::sync::atomic::AtomicBool;

use crate::dataset::Dataset;
use crate::file_system::tests::double::Memory;
use crate::genealogy::{CreationOutcome, Experiment};
use crate::model::Model;
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::tests::double::Double;
use crate::sandboxes::{self, Environment, Host, Launch, SandboxError, Trial};

const LAUNCH: Launch<'static> = Launch {
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

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let dataset = Dataset::read(
        &directory,
        &test_inputs::dataset_in(&directory, directory.path(), &["t1"]),
    )
    .unwrap();
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    outputs_directory.copy_dataset(&dataset).unwrap();
    (directory, outputs_directory)
}

fn host<'a>(docker: &'a Double, outputs_directory: &'a OutputsDirectory) -> Host<'a> {
    Host {
        docker,
        outputs_directory,
    }
}

#[test]
fn a_diversification_of_a_harness_that_is_not_there_is_an_error() {
    let (_directory, outputs_directory) = made();
    let result = sandboxes::run_diversification(
        host(&Double::default(), &outputs_directory),
        &LAUNCH,
        1,
        9,
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(SandboxError::Io(_))));
}

#[test]
fn a_baseline_that_cannot_be_started_in_the_outputs_directory_is_not_carried_out() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_diversification(1, 0).unwrap();
    outputs_directory
        .end_diversification(1, 0, CreationOutcome::Created)
        .unwrap();
    directory
        .create_dir(outputs_directory.running_baseline(5))
        .unwrap();
    let result = sandboxes::run_baseline(
        host(&Double::default(), &outputs_directory),
        &LAUNCH,
        5,
        Experiment {
            child: 3,
            progenitor: 1,
        },
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(SandboxError::Io(_))));
}

#[test]
fn a_trial_of_a_harness_that_is_not_there_is_an_error() {
    let (_directory, outputs_directory) = made();
    let environment = Environment {
        image: test_inputs::IMAGE.to_owned(),
        ..Environment::default()
    };
    let result = sandboxes::run_trial(
        host(&Double::default(), &outputs_directory),
        &LAUNCH,
        Trial {
            harness: 9,
            task: "dataset/t1",
            environment: &environment,
        },
        &AtomicBool::new(false),
        &|_| (),
    );
    assert!(matches!(result, Err(SandboxError::Io(_))));
}
