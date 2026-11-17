use std::collections::BTreeSet;

use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

fn tasks(count: usize) -> Vec<String> {
    (0..count).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64, task_count: usize) -> ArtificialSelection {
    ArtificialSelection {
        budget: 1000,
        growth: 0.0,
        confidence: 99.0,
        tasks: tasks(task_count),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: ProcreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn attempted(genealogy: &mut Genealogy, harness: HarnessId, count: usize, result: AttemptResult) {
    for task in tasks(count) {
        genealogy.start_attempt(harness, &task).unwrap();
        genealogy.end_attempt(harness, &task, result).unwrap();
    }
}

fn two_lines() -> Genealogy {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    attempted(&mut genealogy, 1, 4, AttemptResult::NotCompleted);
    attempted(&mut genealogy, 3, 8, AttemptResult::Completed);
    attempted(&mut genealogy, 2, 4, AttemptResult::Completed);
    attempted(&mut genealogy, 4, 8, AttemptResult::NotCompleted);
    genealogy
}

fn attempt(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<(HarnessId, String)> {
    artificial_selection(seed, task_count)
        .next_action(genealogy, 0)
        .and_then(|action| match action {
            Action::Attempt { harness, task } => Some((harness, task)),
            Action::Procreation { .. } => None,
        })
}

fn attempting(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<HarnessId> {
    attempt(genealogy, seed, task_count).map(|(harness, _)| harness)
}

fn times_attempting(genealogy: &Genealogy, harness: HarnessId, task_count: usize) -> usize {
    (0..200)
        .filter(|seed| attempting(genealogy, *seed, task_count) == Some(harness))
        .count()
}

fn tasks_drawn(genealogy: &Genealogy, task_count: usize) -> BTreeSet<String> {
    (0..200)
        .filter_map(|seed| attempt(genealogy, seed, task_count))
        .map(|(_, task)| task)
        .collect()
}

#[test]
fn an_attempt_is_the_next_action_when_no_procreation_is_due() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assert_eq!(attempting(&genealogy, 0, 20), Some(0));
}

#[test]
fn a_procreation_is_not_an_attempt() {
    let mut genealogy = Genealogy::new();
    attempted(&mut genealogy, 0, 2, AttemptResult::Completed);
    assert_eq!(attempting(&genealogy, 0, 2), None);
}

#[test]
fn an_attempt_takes_the_place_of_a_procreation_that_no_harness_has_attempted_enough_for() {
    assert_eq!(attempting(&Genealogy::new(), 0, 20), Some(0));
}

#[test]
fn a_harness_with_better_results_attempts_a_task_more_often_than_one_with_the_better_clade() {
    let genealogy = two_lines();
    assert!(times_attempting(&genealogy, 2, 20) > times_attempting(&genealogy, 1, 20));
}

#[test]
fn a_harness_with_no_task_left_never_attempts_a_task() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    attempted(&mut genealogy, 0, 2, AttemptResult::Completed);
    assert_eq!(times_attempting(&genealogy, 1, 2), 200);
}

#[test]
fn a_harness_that_is_being_created_never_attempts_a_task() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    attempted(&mut genealogy, 0, 2, AttemptResult::NotCompleted);
    assert_eq!(times_attempting(&genealogy, 0, 20), 200);
}

#[test]
fn a_failed_harness_never_attempts_a_task() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    attempted(&mut genealogy, 0, 2, AttemptResult::NotCompleted);
    assert_eq!(times_attempting(&genealogy, 0, 20), 200);
}

#[test]
fn the_only_task_not_yet_given_is_the_one_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    attempted(&mut genealogy, 0, 4, AttemptResult::Completed);
    assert_eq!(
        tasks_drawn(&genealogy, 5),
        BTreeSet::from(["task-4".to_owned()])
    );
}

#[test]
fn every_task_not_yet_given_can_be_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    attempted(&mut genealogy, 0, 2, AttemptResult::Completed);
    assert_eq!(
        tasks_drawn(&genealogy, 5),
        BTreeSet::from([
            "task-2".to_owned(),
            "task-3".to_owned(),
            "task-4".to_owned()
        ])
    );
}

#[test]
fn a_task_whose_attempt_is_running_is_not_drawn_again() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    genealogy.start_attempt(0, "task-0").unwrap();
    assert_eq!(
        tasks_drawn(&genealogy, 2),
        BTreeSet::from(["task-1".to_owned()])
    );
}

#[test]
fn every_harness_is_given_every_task_exactly_once() {
    let artificial_selection = artificial_selection(3, 5);
    let mut genealogy = Genealogy::new();
    let mut given = Vec::new();
    let mut number = 0;
    while let Some(action) = artificial_selection.next_action(&genealogy, number) {
        match action {
            Action::Procreation { progenitor } => {
                child_of(&mut genealogy, progenitor, ProcreationOutcome::Created)
            }
            Action::Attempt { harness, task } => {
                genealogy.start_attempt(harness, &task).unwrap();
                genealogy
                    .end_attempt(harness, &task, AttemptResult::Completed)
                    .unwrap();
                given.push((harness, task));
            }
        }
        number += 1;
    }
    given.sort();
    let every_pair: Vec<(HarnessId, String)> = (0..2)
        .flat_map(|harness| tasks(5).into_iter().map(move |task| (harness, task)))
        .collect();
    assert_eq!(given, every_pair);
}
