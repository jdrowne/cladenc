use std::collections::BTreeSet;

use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{Genealogy, HarnessId, ProcreationOutcome, Score};

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

fn assess(genealogy: &mut Genealogy, harness: HarnessId, count: usize, score: Score) {
    for task in tasks(count) {
        genealogy.start_assessment(harness, &task).unwrap();
        genealogy.end_assessment(harness, &task, score).unwrap();
    }
}

fn two_lines() -> Genealogy {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    assess(&mut genealogy, 1, 4, Score::Failure);
    assess(&mut genealogy, 3, 8, Score::Success);
    assess(&mut genealogy, 2, 4, Score::Success);
    assess(&mut genealogy, 4, 8, Score::Failure);
    genealogy
}

fn assessment(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<(HarnessId, String)> {
    artificial_selection(seed, task_count)
        .next_action(genealogy, 0)
        .and_then(|action| match action {
            Action::Assessment { harness, task } => Some((harness, task)),
            Action::Procreation { .. } => None,
        })
}

fn assessed(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<HarnessId> {
    assessment(genealogy, seed, task_count).map(|(harness, _)| harness)
}

fn times_assessed(genealogy: &Genealogy, harness: HarnessId, task_count: usize) -> usize {
    (0..200)
        .filter(|seed| assessed(genealogy, *seed, task_count) == Some(harness))
        .count()
}

fn tasks_drawn(genealogy: &Genealogy, task_count: usize) -> BTreeSet<String> {
    (0..200)
        .filter_map(|seed| assessment(genealogy, seed, task_count))
        .map(|(_, task)| task)
        .collect()
}

#[test]
fn an_assessment_is_the_next_action_when_no_procreation_is_due() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assert_eq!(assessed(&genealogy, 0, 20), Some(0));
}

#[test]
fn a_procreation_is_not_an_assessment() {
    assert_eq!(assessed(&Genealogy::new(), 0, 20), None);
}

#[test]
fn a_harness_with_better_results_is_assessed_more_often_than_one_with_the_better_clade() {
    let genealogy = two_lines();
    assert!(times_assessed(&genealogy, 2, 20) > times_assessed(&genealogy, 1, 20));
}

#[test]
fn a_harness_with_no_task_left_is_never_assessed() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    assess(&mut genealogy, 0, 2, Score::Success);
    assert_eq!(times_assessed(&genealogy, 1, 2), 200);
}

#[test]
fn a_harness_that_is_being_created_is_never_assessed() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assess(&mut genealogy, 0, 2, Score::Failure);
    assert_eq!(times_assessed(&genealogy, 0, 20), 200);
}

#[test]
fn a_failed_harness_is_never_assessed() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assess(&mut genealogy, 0, 2, Score::Failure);
    assert_eq!(times_assessed(&genealogy, 0, 20), 200);
}

#[test]
fn the_only_task_not_yet_given_is_the_one_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assess(&mut genealogy, 0, 4, Score::Success);
    assert_eq!(
        tasks_drawn(&genealogy, 5),
        BTreeSet::from(["task-4".to_owned()])
    );
}

#[test]
fn every_task_not_yet_given_can_be_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assess(&mut genealogy, 0, 2, Score::Success);
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
fn a_task_whose_assessment_is_running_is_not_drawn_again() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    genealogy.start_assessment(0, "task-0").unwrap();
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
            Action::Assessment { harness, task } => {
                genealogy.start_assessment(harness, &task).unwrap();
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
