use std::collections::BTreeSet;

use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

fn tasks(count: usize) -> Vec<String> {
    (0..count).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64, task_count: usize) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations: 1000,
        tasks: tasks(task_count),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn trialled(genealogy: &mut Genealogy, harness: HarnessId, count: usize, result: TrialResult) {
    for task in tasks(count) {
        genealogy.start_trial(harness, &task).unwrap();
        genealogy.end_trial(harness, &task, result).unwrap();
    }
}

fn with_two_children() -> Genealogy {
    let mut genealogy = Genealogy::new();
    trialled(&mut genealogy, 0, 5, TrialResult::NotCompleted);
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    genealogy
}

fn trial(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<(HarnessId, String)> {
    artificial_selection(seed, task_count)
        .next_action(genealogy, 0)
        .and_then(|action| match action {
            Action::Trial { harness, task } => Some((harness, task)),
            Action::Procreation { .. } => None,
        })
}

fn trialling(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<HarnessId> {
    trial(genealogy, seed, task_count).map(|(harness, _)| harness)
}

fn times_trialling(genealogy: &Genealogy, harness: HarnessId, task_count: usize) -> usize {
    (0..200)
        .filter(|seed| trialling(genealogy, *seed, task_count) == Some(harness))
        .count()
}

fn tasks_trialled(genealogy: &Genealogy, task_count: usize) -> BTreeSet<String> {
    (0..200)
        .filter_map(|seed| trial(genealogy, seed, task_count))
        .map(|(_, task)| task)
        .collect()
}

#[test]
fn a_trial_is_the_next_action_when_no_procreation_is_due() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    assert_eq!(trialling(&genealogy, 0, 20), Some(0));
}

#[test]
fn a_procreation_is_not_a_trial() {
    let mut genealogy = Genealogy::new();
    trialled(&mut genealogy, 0, 2, TrialResult::Completed);
    assert_eq!(trialling(&genealogy, 0, 2), None);
}

#[test]
fn a_trial_takes_the_place_of_a_procreation_that_no_harness_is_benchmarked_in_full_for() {
    assert_eq!(trialling(&Genealogy::new(), 0, 20), Some(0));
}

#[test]
fn the_harness_with_the_lowest_id_that_has_a_task_not_yet_given_trials_next() {
    let mut genealogy = with_two_children();
    trialled(&mut genealogy, 1, 2, TrialResult::Completed);
    assert_eq!(trialling(&genealogy, 0, 5), Some(1));
}

#[test]
fn a_harness_with_better_results_does_not_trial_ahead_of_one_with_a_lower_id() {
    let mut genealogy = with_two_children();
    trialled(&mut genealogy, 1, 2, TrialResult::NotCompleted);
    trialled(&mut genealogy, 2, 4, TrialResult::Completed);
    assert_eq!(times_trialling(&genealogy, 1, 5), 200);
}

#[test]
fn a_harness_with_no_task_left_never_trials_a_task() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    trialled(&mut genealogy, 0, 2, TrialResult::Completed);
    assert_eq!(times_trialling(&genealogy, 1, 2), 200);
}

#[test]
fn a_harness_that_is_being_created_never_trials_a_task() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    trialled(&mut genealogy, 0, 2, TrialResult::NotCompleted);
    assert_eq!(times_trialling(&genealogy, 0, 20), 200);
}

#[test]
fn a_failed_harness_never_trials_a_task() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    trialled(&mut genealogy, 0, 2, TrialResult::NotCompleted);
    assert_eq!(times_trialling(&genealogy, 0, 20), 200);
}

#[test]
fn the_first_task_not_yet_given_is_the_one_trialled_whatever_the_seed() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    trialled(&mut genealogy, 0, 2, TrialResult::Completed);
    assert_eq!(
        tasks_trialled(&genealogy, 5),
        BTreeSet::from(["task-2".to_owned()])
    );
}

#[test]
fn the_tasks_are_given_in_the_order_they_are_listed_in() {
    let listed = ArtificialSelection {
        tasks: vec!["refactoring/t2".to_owned(), "benchmark/t1".to_owned()],
        ..artificial_selection(0, 0)
    };
    assert_eq!(
        listed.next_action(&Genealogy::new(), 0),
        Some(Action::Trial {
            harness: 0,
            task: "refactoring/t2".to_owned()
        })
    );
}

#[test]
fn a_task_whose_trial_is_running_is_not_given_again() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    genealogy.start_trial(0, "task-0").unwrap();
    assert_eq!(
        tasks_trialled(&genealogy, 2),
        BTreeSet::from(["task-1".to_owned()])
    );
}

#[test]
fn the_action_number_does_not_change_which_trial_is_next() {
    let mut genealogy = with_two_children();
    trialled(&mut genealogy, 1, 2, TrialResult::Completed);
    let first = artificial_selection(7, 5).next_action(&genealogy, 0);
    assert!(
        (1..50).all(|action| artificial_selection(7, 5).next_action(&genealogy, action) == first)
    );
}
