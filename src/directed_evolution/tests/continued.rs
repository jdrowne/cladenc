use std::path::PathBuf;

use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Settings, Started};
use crate::outputs_directory::OutputsDirectory;

const A_TRIAL: [&str; 2] = ["run a harness", "trial-0-benchmark+t2"];
const A_PROCREATION: [&str; 2] = ["run a harness", "procreation-1"];
const NOT_STARTED: Answer = Answer::Exited(125);
const TASK: &str = "benchmark/t2";

fn with_a_selection_task(_: &Memory, settings: &mut Settings) {
    settings.holdout_tasks = vec!["benchmark/t1".to_owned()];
}

fn twice(directory: &Memory, settings: &mut Settings) {
    with_a_selection_task(directory, settings);
    settings.max_procreations = 2;
}

fn working() -> Double {
    Double::default()
        .changing()
        .verifying(|trial| trial.starts_with("trial-1-"))
}

fn unbroken() -> Evolved {
    given::started(working(), twice)
}

fn stopped_at(needles: &[&str]) -> Evolved {
    given::started(working().answering(needles, &[NOT_STARTED]), twice)
}

fn resumed_after_a_stop_at(needles: &[&str]) -> Evolved {
    given::resumed(stopped_at(needles), working())
}

fn stopped_again_at(needles: &[&str]) -> Evolved {
    let still_not_started = working().answering(needles, &[NOT_STARTED]);
    given::resumed(stopped_at(needles), still_not_started)
}

fn stopped_once_at_a_trial() -> Evolved {
    let docker = working().answering(&A_TRIAL, &[NOT_STARTED]);
    given::started(docker, with_a_selection_task)
}

fn resumed_with_something_left(
    needles: &[&str],
    place: impl FnOnce(&OutputsDirectory) -> PathBuf,
) -> Evolved {
    let stopped = stopped_at(needles);
    let left = place(&stopped.outputs_directory()).join("leftover");
    stopped.files().write(left, "left\n").unwrap();
    given::resumed(stopped, working())
}

fn resumed_with_something_left_by_a_trial() -> Evolved {
    resumed_with_something_left(&A_TRIAL, |outputs_directory| {
        outputs_directory.running_trial(0, TASK)
    })
}

fn resumed_with_something_left_by_a_procreation() -> Evolved {
    resumed_with_something_left(&A_PROCREATION, |outputs_directory| {
        outputs_directory.running_procreation(1).join("child")
    })
}

fn journal_of(evolved: &Evolved) -> String {
    test_inputs::without_seconds(&evolved.journal())
}

#[test]
fn a_container_that_ends_with_status_125_at_a_trial_stops_the_directed_evolution_naming_it() {
    assert!(matches!(
        stopped_at(&A_TRIAL).result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { harness: 0, task },
            error: ContainerError::Docker(_),
        }) if task == TASK
    ));
}

#[test]
fn a_directed_evolution_a_trial_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    assert_eq!(
        journal_of(&resumed_after_a_stop_at(&A_TRIAL)),
        journal_of(&unbroken())
    );
}

#[test]
fn a_container_that_ends_with_status_125_at_a_procreation_stops_the_directed_evolution_naming_it() {
    assert!(matches!(
        stopped_at(&A_PROCREATION).result,
        Err(DirectedEvolutionError::Action {
            action: Started::Procreation {
                child: 1,
                progenitor: 0
            },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_a_procreation_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    assert_eq!(
        journal_of(&resumed_after_a_stop_at(&A_PROCREATION)),
        journal_of(&unbroken())
    );
}

#[test]
fn a_directed_evolution_resumed_while_docker_still_does_not_start_the_container_stops_again() {
    assert!(matches!(
        stopped_again_at(&A_TRIAL).result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { .. },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_that_stopped_again_continues_to_the_same_journal_once_resumed_once_more() {
    let resumed = given::resumed(stopped_again_at(&A_TRIAL), working());
    assert_eq!(journal_of(&resumed), journal_of(&unbroken()));
}

#[test]
fn a_resumed_directed_evolution_returns_the_harness_an_unbroken_directed_evolution_returns() {
    assert_eq!(
        [
            resumed_after_a_stop_at(&A_TRIAL).best_harness(),
            unbroken().best_harness()
        ],
        [Some(1), Some(1)]
    );
}

#[test]
fn a_resumed_directed_evolution_returns_the_directory_of_the_best_harness_in_its_outputs_directory()
{
    let resumed = resumed_after_a_stop_at(&A_TRIAL);
    let best = resumed.outputs().join("harnesses/1");
    assert_eq!(resumed.result.ok().map(|found| found.directory), Some(best));
}

#[test]
fn a_directed_evolution_that_an_action_stops_says_what_became_of_the_action() {
    assert!(stopped_at(&A_TRIAL).said.contains(
        "cladenc: the trial of harness 0 on task benchmark/t2 could not be carried out: Docker \
         did not start its container, and "
    ));
}

#[test]
fn a_directed_evolution_that_an_action_stops_says_last_that_it_is_stopping() {
    assert_eq!(
        stopped_at(&A_TRIAL).lines().last().map(String::as_str),
        Some("cladenc: stopping the directed evolution, with 0 containers running")
    );
}

#[test]
fn a_resumed_directed_evolution_says_first_how_far_the_directed_evolution_had_got() {
    let resumed = resumed_after_a_stop_at(&A_TRIAL);
    assert!(resumed.said.starts_with(&format!(
        "cladenc: continuing the directed evolution in {}, which has started ",
        resumed.outputs().display()
    )));
}

#[test]
fn a_resumed_directed_evolution_counts_the_trials_started_before_it_stopped_when_it_finishes() {
    let resumed = given::resumed(stopped_once_at_a_trial(), working());
    assert!(resumed.lines().last().is_some_and(|last| {
        last.starts_with("cladenc: the directed evolution finished with 4 trials started and ")
    }));
}

#[test]
fn a_journal_cut_before_the_finished_record_of_an_action_in_place_continues_to_the_same_journal() {
    let evolved = unbroken();
    let whole = evolved.journal();
    let kept = whole.lines().count() - 2;
    let cut: String = whole.split_inclusive('\n').take(kept).collect();
    evolved
        .files()
        .write(journal::path(&evolved.outputs()), cut)
        .unwrap();
    let resumed = given::resumed(evolved, working());
    assert_eq!(journal_of(&resumed), test_inputs::without_seconds(&whole));
}

#[test]
fn a_directed_evolution_is_resumed_without_the_directory_its_benchmark_was_given_in() {
    let stopped = stopped_at(&A_PROCREATION);
    stopped
        .files()
        .remove_dir_all(stopped.files().path().join("benchmark"))
        .unwrap();
    assert!(given::resumed(stopped, working()).result.is_ok());
}

#[test]
fn what_a_trial_left_where_it_was_running_is_gone_before_it_is_started_again() {
    let resumed = resumed_with_something_left_by_a_trial();
    let kept = resumed.outputs_directory().trial(0, TASK);
    assert!(!resumed.files().exists(kept.join("leftover")));
}

#[test]
fn a_trial_that_is_started_again_ends_with_its_result_in_place() {
    let resumed = resumed_with_something_left_by_a_trial();
    let kept = resumed.outputs_directory().trial(0, TASK);
    assert!(resumed.files().is_file(kept.join("result")));
}

#[test]
fn what_a_procreation_left_where_it_was_running_is_gone_before_it_is_started_again() {
    let resumed = resumed_with_something_left_by_a_procreation();
    let child = resumed.outputs_directory().harness(1);
    assert!(!resumed.files().exists(child.join("leftover")));
}

#[test]
fn a_procreation_that_is_started_again_ends_with_its_child_in_place() {
    let resumed = resumed_with_something_left_by_a_procreation();
    let child = resumed.outputs_directory().harness(1);
    assert!(resumed.files().is_file(child.join("changed")));
}
