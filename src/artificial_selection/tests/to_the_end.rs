use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy, HarnessId};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection() -> ArtificialSelection {
    ArtificialSelection {
        max_procreations: 3,
        tasks: tasks(),
        seed: 3,
    }
}

fn actions_to_the_end(outcome: CreationOutcome, result: AttemptResult) -> Vec<Action> {
    let artificial_selection = artificial_selection();
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    let mut actions = Vec::new();
    while let Some(action) = artificial_selection.next_action(&genealogy, actions.len() as u64) {
        match &action {
            Action::Procreation { progenitor } => {
                let child = genealogy.start_procreation(*progenitor).unwrap();
                genealogy.end_procreation(child, outcome).unwrap();
            }
            Action::Attempt { harness, task } => {
                genealogy.start_attempt(*harness, task).unwrap();
                genealogy.end_attempt(*harness, task, result).unwrap();
            }
        }
        actions.push(action);
    }
    actions
}

fn procreations(actions: &[Action]) -> usize {
    actions
        .iter()
        .filter(|action| matches!(action, Action::Procreation { .. }))
        .count()
}

fn attempts(actions: &[Action]) -> Vec<(HarnessId, String)> {
    let mut attempts: Vec<(HarnessId, String)> = actions
        .iter()
        .filter_map(|action| match action {
            Action::Attempt { harness, task } => Some((*harness, task.clone())),
            Action::Procreation { .. } => None,
        })
        .collect();
    attempts.sort();
    attempts
}

fn every_task_for_each_of(harnesses: &[HarnessId]) -> Vec<(HarnessId, String)> {
    harnesses
        .iter()
        .flat_map(|harness| tasks().into_iter().map(move |task| (*harness, task)))
        .collect()
}

fn shape(actions: &[Action]) -> Vec<String> {
    actions
        .iter()
        .map(|action| match action {
            Action::Attempt { harness, .. } => format!("attempt of {harness}"),
            Action::Procreation { .. } => "procreation".to_owned(),
        })
        .collect()
}

#[test]
fn exactly_as_many_procreations_start_as_the_most_there_may_be() {
    let actions = actions_to_the_end(CreationOutcome::Created, AttemptResult::Completed);
    assert_eq!(procreations(&actions), 3);
}

#[test]
fn a_procreation_that_fails_is_one_of_the_most_there_may_be() {
    let actions = actions_to_the_end(CreationOutcome::Failed, AttemptResult::Completed);
    assert_eq!(procreations(&actions), 3);
}

#[test]
fn at_the_end_every_created_harness_has_attempted_every_task_exactly_once() {
    let actions = actions_to_the_end(CreationOutcome::Created, AttemptResult::Completed);
    assert_eq!(attempts(&actions), every_task_for_each_of(&[0, 1, 2, 3]));
}

#[test]
fn a_harness_that_completes_none_of_its_tasks_still_attempts_every_task() {
    let actions = actions_to_the_end(CreationOutcome::Created, AttemptResult::NotCompleted);
    assert_eq!(attempts(&actions), every_task_for_each_of(&[0, 1, 2, 3]));
}

#[test]
fn a_harness_that_was_not_created_attempts_nothing() {
    let actions = actions_to_the_end(CreationOutcome::Failed, AttemptResult::Completed);
    assert_eq!(attempts(&actions), every_task_for_each_of(&[0]));
}

#[test]
fn each_child_is_benchmarked_in_full_before_the_next_procreation_starts() {
    let actions = actions_to_the_end(CreationOutcome::Created, AttemptResult::Completed);
    assert_eq!(
        shape(&actions),
        [
            "attempt of 0",
            "attempt of 0",
            "procreation",
            "attempt of 1",
            "attempt of 1",
            "procreation",
            "attempt of 2",
            "attempt of 2",
            "procreation",
            "attempt of 3",
            "attempt of 3",
        ]
    );
}

#[test]
fn every_procreation_still_starts_when_each_one_fails() {
    let actions = actions_to_the_end(CreationOutcome::Failed, AttemptResult::Completed);
    assert_eq!(
        shape(&actions),
        [
            "attempt of 0",
            "attempt of 0",
            "procreation",
            "procreation",
            "procreation",
        ]
    );
}
