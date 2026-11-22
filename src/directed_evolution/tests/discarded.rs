use std::fs;

use tempfile::TempDir;

use crate::directed_evolution::action;
use crate::genealogy::CreationOutcome;
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "benchmark/t1";

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

#[test]
fn what_a_procreation_left_is_discarded_before_it_is_carried_out_again() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    outputs_directory
        .end_procreation(1, 0, CreationOutcome::Created)
        .unwrap();
    let procreation = Started::Procreation {
        child: 1,
        progenitor: 0,
    };
    action::discard(&outputs_directory, &procreation).unwrap();
    assert_eq!(
        [
            outputs_directory.harness(1).exists(),
            outputs_directory.procreation(1).exists(),
            outputs_directory.attempts(1).exists(),
        ],
        [false, false, false]
    );
}

#[test]
fn what_a_baseline_left_is_discarded_before_it_is_carried_out_again() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_baseline(2, 0).unwrap();
    let baseline = Started::Baseline {
        harness: 2,
        child: 1,
        progenitor: 0,
    };
    action::discard(&outputs_directory, &baseline).unwrap();
    assert!(!outputs_directory.running_baseline(2).exists());
}

#[test]
fn what_an_attempt_left_is_discarded_before_it_is_carried_out_again() {
    let (_directory, outputs_directory) = made();
    fs::create_dir(outputs_directory.running_attempt(0, TASK)).unwrap();
    let attempt = Started::Attempt {
        harness: 0,
        task: TASK.to_owned(),
    };
    action::discard(&outputs_directory, &attempt).unwrap();
    assert!(!outputs_directory.running_attempt(0, TASK).exists());
}

#[test]
fn the_harness_that_procreated_is_kept_when_what_its_procreation_left_is_discarded() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    let procreation = Started::Procreation {
        child: 1,
        progenitor: 0,
    };
    action::discard(&outputs_directory, &procreation).unwrap();
    assert_eq!(
        [
            outputs_directory.running_procreation(1).exists(),
            outputs_directory.harness(0).exists(),
        ],
        [false, true]
    );
}
