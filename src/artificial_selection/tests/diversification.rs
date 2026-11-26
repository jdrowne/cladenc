use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_diversifications: 1000,
        tasks: tasks(),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_diversification(progenitor).unwrap();
    genealogy.end_diversification(child, outcome).unwrap();
}

fn trial(genealogy: &mut Genealogy, harness: HarnessId, count: usize, result: TrialResult) {
    for task in tasks().iter().take(count) {
        genealogy.start_trial(harness, task).unwrap();
        genealogy.end_trial(harness, task, result).unwrap();
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    trial(&mut genealogy, 0, 20, TrialResult::NotCompleted);
    genealogy
}

fn with_a_child_that_trialled(count: usize) -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    trial(&mut genealogy, 1, count, TrialResult::Completed);
    genealogy
}

fn benchmark(genealogy: &mut Genealogy, harness: HarnessId, completed: usize) {
    for (number, task) in tasks().iter().enumerate() {
        let result = if number < completed {
            TrialResult::Completed
        } else {
            TrialResult::NotCompleted
        };
        genealogy.start_trial(harness, task).unwrap();
        genealogy.end_trial(harness, task, result).unwrap();
    }
}

fn two_lines() -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmark(&mut genealogy, 1, 10);
    benchmark(&mut genealogy, 2, 12);
    child_of(&mut genealogy, 1, CreationOutcome::Created);
    child_of(&mut genealogy, 1, CreationOutcome::Created);
    child_of(&mut genealogy, 2, CreationOutcome::Created);
    child_of(&mut genealogy, 2, CreationOutcome::Created);
    benchmark(&mut genealogy, 3, 12);
    benchmark(&mut genealogy, 4, 12);
    benchmark(&mut genealogy, 5, 0);
    benchmark(&mut genealogy, 6, 0);
    genealogy
}

fn times_a_diversification_is_next(genealogy: &Genealogy) -> usize {
    (0..200)
        .filter(|seed| {
            matches!(
                artificial_selection(*seed).next_action(genealogy, 0),
                Some(Action::Diversification { .. })
            )
        })
        .count()
}

fn times_progenitor(genealogy: &Genealogy, progenitor: HarnessId) -> usize {
    (0..200)
        .filter(|seed| {
            artificial_selection(*seed).next_action(genealogy, 0)
                == Some(Action::Diversification { progenitor })
        })
        .count()
}

#[test]
fn a_trial_is_the_next_action_while_a_child_has_a_task_not_yet_given() {
    assert_eq!(
        artificial_selection(0).next_action(&with_a_child_that_trialled(0), 0),
        Some(Action::Trial {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_harness_with_the_better_clade_is_the_progenitor_more_often_than_one_with_better_results() {
    let genealogy = two_lines();
    assert!(times_progenitor(&genealogy, 1) > times_progenitor(&genealogy, 2));
}

#[test]
fn no_diversification_starts_while_a_child_that_was_just_created_has_trialled_nothing() {
    assert_eq!(
        times_a_diversification_is_next(&with_a_child_that_trialled(0)),
        0
    );
}

#[test]
fn no_diversification_starts_while_a_child_has_trialled_every_task_but_one() {
    assert_eq!(
        times_a_diversification_is_next(&with_a_child_that_trialled(19)),
        0
    );
}

#[test]
fn no_diversification_starts_while_a_trial_of_a_child_given_every_task_is_running() {
    let mut genealogy = with_a_child_that_trialled(19);
    genealogy.start_trial(1, "task-19").unwrap();
    assert_eq!(times_a_diversification_is_next(&genealogy), 0);
}

#[test]
fn a_child_can_be_the_progenitor_once_it_has_trialled_every_task() {
    assert!(times_progenitor(&with_a_child_that_trialled(20), 1) > 0);
}

#[test]
fn no_diversification_starts_while_a_child_is_being_created() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(times_a_diversification_is_next(&genealogy), 0);
}

#[test]
fn a_failed_harness_is_never_the_progenitor() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    assert_eq!(times_progenitor(&genealogy, 0), 200);
}

#[test]
fn a_diversification_is_the_next_action_once_no_task_is_left_to_give() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    trial(&mut genealogy, 0, 20, TrialResult::Completed);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Diversification { progenitor: 0 })
    );
}

#[test]
fn a_harness_that_completed_no_task_can_be_the_progenitor() {
    assert_eq!(
        artificial_selection(0).next_action(&with_the_primogenitor_benchmarked_in_full(), 0),
        Some(Action::Diversification { progenitor: 0 })
    );
}

#[test]
fn a_harness_whose_trials_ran_past_their_time_limit_can_be_the_progenitor() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    trial(&mut genealogy, 0, 20, TrialResult::TimedOut);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Diversification { progenitor: 0 })
    );
}

#[test]
fn a_trial_takes_the_place_of_a_diversification_no_harness_has_trialled_enough_for() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    trial(&mut genealogy, 0, 19, TrialResult::Completed);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Trial {
            harness: 0,
            task: "task-19".to_owned()
        })
    );
}
