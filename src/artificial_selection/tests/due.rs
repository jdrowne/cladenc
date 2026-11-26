use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_diversifications: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_diversifications,
        tasks: tasks(),
        seed: 0,
    }
}

fn benchmark_in_full(genealogy: &mut Genealogy, harness: HarnessId) {
    for task in tasks() {
        genealogy.start_trial(harness, &task).unwrap();
        genealogy
            .end_trial(harness, &task, TrialResult::Fail)
            .unwrap();
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    benchmark_in_full(&mut genealogy, 0);
    genealogy
}

fn with_a_child(outcome: CreationOutcome) -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy.end_diversification(child, outcome).unwrap();
    genealogy
}

fn is_a_diversification(action: Option<Action>) -> bool {
    matches!(action, Some(Action::Diversification { .. }))
}

#[test]
fn a_diversification_is_due_once_every_created_harness_is_benchmarked_in_full() {
    let mut genealogy = with_a_child(CreationOutcome::Created);
    benchmark_in_full(&mut genealogy, 1);
    assert!(is_a_diversification(
        artificial_selection(100).next_action(&genealogy, 0)
    ));
}

#[test]
fn a_failed_harness_does_not_hold_back_a_diversification() {
    assert_eq!(
        artificial_selection(100).next_action(&with_a_child(CreationOutcome::Failed), 0),
        Some(Action::Diversification { progenitor: 0 })
    );
}

#[test]
fn a_created_harness_with_a_task_not_yet_given_trials_it_before_a_diversification_starts() {
    assert_eq!(
        artificial_selection(100).next_action(&with_a_child(CreationOutcome::Created), 0),
        Some(Action::Trial {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn no_diversification_is_due_while_a_trial_is_running() {
    let mut genealogy = with_a_child(CreationOutcome::Created);
    genealogy.start_trial(1, "task-0").unwrap();
    genealogy.start_trial(1, "task-1").unwrap();
    assert_eq!(artificial_selection(100).next_action(&genealogy, 0), None);
}

#[test]
fn a_trial_is_the_next_action_once_the_most_diversifications_have_started() {
    assert_eq!(
        artificial_selection(1).next_action(&with_a_child(CreationOutcome::Created), 0),
        Some(Action::Trial {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_diversification_is_still_the_next_action_when_one_fewer_than_the_most_have_started() {
    let mut genealogy = with_a_child(CreationOutcome::Created);
    benchmark_in_full(&mut genealogy, 1);
    assert!(is_a_diversification(
        artificial_selection(2).next_action(&genealogy, 0)
    ));
}

#[test]
fn a_diversification_that_failed_counts_among_those_started() {
    assert_eq!(
        artificial_selection(1).next_action(&with_a_child(CreationOutcome::Failed), 0),
        None
    );
}

#[test]
fn no_diversification_is_due_once_the_most_have_started() {
    let mut genealogy = with_a_child(CreationOutcome::Created);
    benchmark_in_full(&mut genealogy, 1);
    assert_eq!(artificial_selection(1).next_action(&genealogy, 0), None);
}
