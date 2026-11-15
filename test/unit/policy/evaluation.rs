use std::collections::BTreeSet;

use cladenc::genealogy::{EvaluationOutcome, Genealogy, HarnessId, ProcreationOutcome};
use cladenc::policy::{Job, Policy};

fn tasks(count: usize) -> Vec<String> {
    (0..count).map(|number| format!("task-{number}")).collect()
}

fn policy(seed: u64, task_count: usize) -> Policy {
    Policy {
        budget: 1000,
        growth: 0.0,
        confidence: 99.0,
        tasks: tasks(task_count),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, parent: HarnessId, outcome: ProcreationOutcome) {
    let child = genealogy.start_procreation(parent).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn evaluate(
    genealogy: &mut Genealogy,
    harness: HarnessId,
    count: usize,
    outcome: EvaluationOutcome,
) {
    for task in tasks(count) {
        genealogy.start_evaluation(harness, &task).unwrap();
        genealogy.end_evaluation(harness, &task, outcome).unwrap();
    }
}

fn two_lines() -> Genealogy {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 1, 4, EvaluationOutcome::Failure);
    evaluate(&mut genealogy, 3, 8, EvaluationOutcome::Success);
    evaluate(&mut genealogy, 2, 4, EvaluationOutcome::Success);
    evaluate(&mut genealogy, 4, 8, EvaluationOutcome::Failure);
    genealogy
}

fn evaluation(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<(HarnessId, String)> {
    policy(seed, task_count)
        .next_job(genealogy, 0)
        .and_then(|job| match job {
            Job::Evaluation { harness, task } => Some((harness, task)),
            Job::Procreation { .. } => None,
        })
}

fn evaluated(genealogy: &Genealogy, seed: u64, task_count: usize) -> Option<HarnessId> {
    evaluation(genealogy, seed, task_count).map(|(harness, _)| harness)
}

fn times_evaluated(genealogy: &Genealogy, harness: HarnessId, task_count: usize) -> usize {
    (0..200)
        .filter(|seed| evaluated(genealogy, *seed, task_count) == Some(harness))
        .count()
}

fn tasks_drawn(genealogy: &Genealogy, task_count: usize) -> BTreeSet<String> {
    (0..200)
        .filter_map(|seed| evaluation(genealogy, seed, task_count))
        .map(|(_, task)| task)
        .collect()
}

#[test]
fn an_evaluation_is_the_next_job_when_no_procreation_is_due() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assert_eq!(evaluated(&genealogy, 0, 20), Some(0));
}

#[test]
fn a_procreation_is_not_an_evaluation() {
    assert_eq!(evaluated(&Genealogy::new(), 0, 20), None);
}

#[test]
fn a_harness_with_better_results_is_evaluated_more_often_than_one_with_the_better_clade() {
    let genealogy = two_lines();
    assert!(times_evaluated(&genealogy, 2, 20) > times_evaluated(&genealogy, 1, 20));
}

#[test]
fn a_harness_with_no_task_left_is_never_evaluated() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 0, 2, EvaluationOutcome::Success);
    assert_eq!(times_evaluated(&genealogy, 1, 2), 200);
}

#[test]
fn a_harness_that_is_being_created_is_never_evaluated() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    evaluate(&mut genealogy, 0, 2, EvaluationOutcome::Failure);
    assert_eq!(times_evaluated(&genealogy, 0, 20), 200);
}

#[test]
fn a_failed_harness_is_never_evaluated() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    evaluate(&mut genealogy, 0, 2, EvaluationOutcome::Failure);
    assert_eq!(times_evaluated(&genealogy, 0, 20), 200);
}

#[test]
fn the_only_task_not_yet_given_is_the_one_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    evaluate(&mut genealogy, 0, 4, EvaluationOutcome::Success);
    assert_eq!(
        tasks_drawn(&genealogy, 5),
        BTreeSet::from(["task-4".to_owned()])
    );
}

#[test]
fn every_task_not_yet_given_can_be_drawn() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    evaluate(&mut genealogy, 0, 2, EvaluationOutcome::Success);
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
fn a_task_whose_evaluation_is_running_is_not_drawn_again() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    genealogy.start_evaluation(0, "task-0").unwrap();
    assert_eq!(
        tasks_drawn(&genealogy, 2),
        BTreeSet::from(["task-1".to_owned()])
    );
}

#[test]
fn every_harness_is_given_every_task_exactly_once() {
    let policy = policy(3, 5);
    let mut genealogy = Genealogy::new();
    let mut given = Vec::new();
    let mut number = 0;
    while let Some(job) = policy.next_job(&genealogy, number) {
        match job {
            Job::Procreation { parent } => {
                child_of(&mut genealogy, parent, ProcreationOutcome::Created)
            }
            Job::Evaluation { harness, task } => {
                genealogy.start_evaluation(harness, &task).unwrap();
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
