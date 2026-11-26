use std::path::Path;

use crate::benchmark::Benchmark;
use crate::file_system::tests::double::Memory;
use crate::genealogy::TrialResult;
use crate::outputs_directory::OutputsDirectory;

const SHOWN: &str = "benchmark/t1";
const HELD_OUT: &str = "benchmark/t2";

fn holding_out_one_task(directory: &Memory, path: &Path) -> OutputsDirectory {
    OutputsDirectory::at(directory.shared(), path).holding_out(&[HELD_OUT.to_owned()])
}

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap()
    .holding_out(&[HELD_OUT.to_owned()]);
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&directory, &benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn trialled(tasks: &[&str]) -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    for task in tasks {
        outputs_directory.start_trial(0, task).unwrap();
        outputs_directory
            .end_trial(0, task, TrialResult::Pass)
            .unwrap();
    }
    (directory, outputs_directory)
}

fn with_the_trial_on_the_holdout_task_discarded() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = trialled(&[SHOWN, HELD_OUT]);
    outputs_directory.discard_trial(0, HELD_OUT).unwrap();
    (directory, outputs_directory)
}

fn names_in(files: &Memory, path: &Path) -> Vec<String> {
    files.names_in(path)
}

#[test]
fn the_trial_on_a_holdout_task_is_kept_under_holdout() {
    assert_eq!(
        holding_out_one_task(&Memory::default(), Path::new("/outputs")).trial(3, HELD_OUT),
        Path::new("/outputs/holdout/3/benchmark/t2")
    );
}

#[test]
fn the_trial_on_a_task_that_is_not_held_out_is_kept_under_trials() {
    assert_eq!(
        holding_out_one_task(&Memory::default(), Path::new("/outputs")).trial(3, SHOWN),
        Path::new("/outputs/trials/3/benchmark/t1")
    );
}

#[test]
fn an_outputs_directory_told_of_no_holdout_task_keeps_every_trial_under_trials() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs")).trial(3, HELD_OUT),
        Path::new("/outputs/trials/3/benchmark/t2")
    );
}

#[test]
fn the_trials_of_a_harness_at_the_holdout_tasks_are_kept_in_a_directory_named_after_it() {
    assert_eq!(
        holding_out_one_task(&Memory::default(), Path::new("/outputs")).holdout(3),
        Path::new("/outputs/holdout/3")
    );
}

#[test]
fn the_directory_a_progenitor_is_given_is_the_one_of_its_trials_that_are_not_held_out() {
    assert_eq!(
        holding_out_one_task(&Memory::default(), Path::new("/outputs")).trials(3),
        Path::new("/outputs/trials/3")
    );
}

#[test]
fn an_outputs_directory_told_of_its_holdout_tasks_is_at_the_same_path() {
    assert_eq!(
        holding_out_one_task(&Memory::default(), Path::new("/outputs")).path(),
        Path::new("/outputs")
    );
}

#[test]
fn a_new_outputs_directory_holds_no_trial_on_a_holdout_task() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        names_in(&directory, &outputs_directory.path().join("holdout")),
        Vec::<String>::new()
    );
}

#[test]
fn a_started_trial_on_a_holdout_task_holds_the_instruction_of_its_task() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_trial(0, HELD_OUT).unwrap();
    assert_eq!(
        directory
            .read_to_string(
                outputs_directory
                    .running_trial(0, HELD_OUT)
                    .join("instruction")
            )
            .unwrap(),
        "Finish the work of t2.\n"
    );
}

#[test]
fn a_trial_on_a_holdout_task_that_ended_is_kept_under_holdout_with_its_result() {
    let (directory, outputs_directory) = trialled(&[HELD_OUT]);
    assert_eq!(
        directory
            .read_to_string(
                outputs_directory
                    .path()
                    .join("holdout/0/benchmark/t2/result")
            )
            .unwrap(),
        "pass\n"
    );
}

#[test]
fn a_trial_on_a_holdout_task_leaves_nothing_in_the_directory_a_progenitor_is_given() {
    let (directory, outputs_directory) = trialled(&[HELD_OUT]);
    assert_eq!(
        names_in(&directory, &outputs_directory.trials(0)),
        Vec::<String>::new()
    );
}

#[test]
fn the_directory_a_progenitor_is_given_holds_the_tasks_that_are_not_held_out_and_no_other() {
    let (directory, outputs_directory) = trialled(&[SHOWN, HELD_OUT]);
    assert_eq!(
        names_in(&directory, &outputs_directory.trials(0).join("benchmark")),
        ["t1"]
    );
}

#[test]
fn the_trials_on_the_holdout_tasks_are_kept_apart_by_benchmark_and_task() {
    let (directory, outputs_directory) = trialled(&[SHOWN, HELD_OUT]);
    assert_eq!(
        names_in(&directory, &outputs_directory.holdout(0).join("benchmark")),
        ["t2"]
    );
}

#[test]
fn discarding_a_trial_on_a_holdout_task_that_ended_leaves_no_directory_of_the_trial() {
    let (directory, outputs_directory) = with_the_trial_on_the_holdout_task_discarded();
    assert!(!directory.exists(outputs_directory.path().join("holdout/0/benchmark/t2")));
}

#[test]
fn discarding_a_trial_on_a_holdout_task_leaves_the_trials_that_are_not_held_out() {
    let (directory, outputs_directory) = with_the_trial_on_the_holdout_task_discarded();
    assert!(directory.is_file(outputs_directory.trial(0, SHOWN).join("result")));
}

#[test]
fn a_discarded_trial_on_a_holdout_task_can_be_started_again() {
    let (_directory, outputs_directory) = with_the_trial_on_the_holdout_task_discarded();
    assert!(outputs_directory.start_trial(0, HELD_OUT).is_ok());
}
