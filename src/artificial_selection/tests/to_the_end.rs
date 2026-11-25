use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

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

fn actions_to_the_end(outcome: CreationOutcome, result: TrialResult) -> Vec<Action> {
    let artificial_selection = artificial_selection();
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    let mut actions = Vec::new();
    while let Some(action) = artificial_selection.next_action(&genealogy, actions.len() as u64) {
        match &action {
            Action::Procreation { progenitor } => {
                let child = genealogy.start_procreation(*progenitor).unwrap();
                genealogy.end_procreation(child, outcome).unwrap();
            }
            Action::Trial { harness, task } => {
                genealogy.start_trial(*harness, task).unwrap();
                genealogy.end_trial(*harness, task, result).unwrap();
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

fn trials(actions: &[Action]) -> Vec<(HarnessId, String)> {
    let mut trials: Vec<(HarnessId, String)> = actions
        .iter()
        .filter_map(|action| match action {
            Action::Trial { harness, task } => Some((*harness, task.clone())),
            Action::Procreation { .. } => None,
        })
        .collect();
    trials.sort();
    trials
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
            Action::Trial { harness, .. } => format!("trial of {harness}"),
            Action::Procreation { .. } => "procreation".to_owned(),
        })
        .collect()
}

#[test]
fn exactly_as_many_procreations_start_as_the_most_there_may_be() {
    let actions = actions_to_the_end(CreationOutcome::Created, TrialResult::Completed);
    assert_eq!(procreations(&actions), 3);
}

#[test]
fn a_procreation_that_fails_is_one_of_the_most_there_may_be() {
    let actions = actions_to_the_end(CreationOutcome::Failed, TrialResult::Completed);
    assert_eq!(procreations(&actions), 3);
}

#[test]
fn at_the_end_every_created_harness_has_trialled_every_task_exactly_once() {
    let actions = actions_to_the_end(CreationOutcome::Created, TrialResult::Completed);
    assert_eq!(trials(&actions), every_task_for_each_of(&[0, 1, 2, 3]));
}

#[test]
fn a_harness_that_completes_none_of_its_tasks_still_trials_every_task() {
    let actions = actions_to_the_end(CreationOutcome::Created, TrialResult::NotCompleted);
    assert_eq!(trials(&actions), every_task_for_each_of(&[0, 1, 2, 3]));
}

#[test]
fn a_harness_that_was_not_created_trials_nothing() {
    let actions = actions_to_the_end(CreationOutcome::Failed, TrialResult::Completed);
    assert_eq!(trials(&actions), every_task_for_each_of(&[0]));
}

#[test]
fn each_child_is_benchmarked_in_full_before_the_next_procreation_starts() {
    let actions = actions_to_the_end(CreationOutcome::Created, TrialResult::Completed);
    assert_eq!(
        shape(&actions),
        [
            "trial of 0",
            "trial of 0",
            "procreation",
            "trial of 1",
            "trial of 1",
            "procreation",
            "trial of 2",
            "trial of 2",
            "procreation",
            "trial of 3",
            "trial of 3",
        ]
    );
}

#[test]
fn every_procreation_still_starts_when_each_one_fails() {
    let actions = actions_to_the_end(CreationOutcome::Failed, TrialResult::Completed);
    assert_eq!(
        shape(&actions),
        [
            "trial of 0",
            "trial of 0",
            "procreation",
            "procreation",
            "procreation",
        ]
    );
}
