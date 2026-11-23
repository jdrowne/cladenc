use crate::directed_evolution::action;
use crate::file_system::tests::double::Memory;
use crate::genealogy::CreationOutcome;
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "benchmark/t1";

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

#[test]
fn what_a_procreation_left_is_discarded_before_it_is_carried_out_again() {
    let (directory, outputs_directory) = made();
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
            directory.exists(outputs_directory.harness(1)),
            directory.exists(outputs_directory.procreation(1)),
            directory.exists(outputs_directory.attempts(1)),
        ],
        [false, false, false]
    );
}

#[test]
fn what_a_baseline_left_is_discarded_before_it_is_carried_out_again() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_baseline(2, 0).unwrap();
    let baseline = Started::Baseline {
        harness: 2,
        child: 1,
        progenitor: 0,
    };
    action::discard(&outputs_directory, &baseline).unwrap();
    assert!(!directory.exists(outputs_directory.running_baseline(2)));
}

#[test]
fn what_an_attempt_left_is_discarded_before_it_is_carried_out_again() {
    let (directory, outputs_directory) = made();
    directory
        .create_dir(outputs_directory.running_attempt(0, TASK))
        .unwrap();
    let attempt = Started::Attempt {
        harness: 0,
        task: TASK.to_owned(),
    };
    action::discard(&outputs_directory, &attempt).unwrap();
    assert!(!directory.exists(outputs_directory.running_attempt(0, TASK)));
}

#[test]
fn the_harness_that_procreated_is_kept_when_what_its_procreation_left_is_discarded() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_procreation(1, 0).unwrap();
    let procreation = Started::Procreation {
        child: 1,
        progenitor: 0,
    };
    action::discard(&outputs_directory, &procreation).unwrap();
    assert_eq!(
        [
            directory.exists(outputs_directory.running_procreation(1)),
            directory.exists(outputs_directory.harness(0)),
        ],
        [false, true]
    );
}
