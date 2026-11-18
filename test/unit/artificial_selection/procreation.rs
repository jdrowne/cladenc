use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations: 1000,
        growth: 1.0,
        tasks: tasks(),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: ProcreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn attempt(genealogy: &mut Genealogy, harness: HarnessId, count: usize, result: AttemptResult) {
    for task in tasks().iter().take(count) {
        genealogy.start_attempt(harness, task).unwrap();
        genealogy.end_attempt(harness, task, result).unwrap();
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().holding_out(&tasks());
    attempt(&mut genealogy, 0, 20, AttemptResult::NotCompleted);
    genealogy
}

fn with_a_child_that_attempted(count: usize) -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    attempt(&mut genealogy, 1, count, AttemptResult::Completed);
    genealogy
}

fn two_lines() -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    attempt(&mut genealogy, 1, 20, AttemptResult::NotCompleted);
    attempt(&mut genealogy, 2, 20, AttemptResult::Completed);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    attempt(&mut genealogy, 3, 19, AttemptResult::Completed);
    attempt(&mut genealogy, 4, 19, AttemptResult::Completed);
    attempt(&mut genealogy, 5, 19, AttemptResult::NotCompleted);
    attempt(&mut genealogy, 6, 19, AttemptResult::NotCompleted);
    genealogy
}

fn times_progenitor(genealogy: &Genealogy, progenitor: HarnessId) -> usize {
    (0..200)
        .filter(|seed| {
            artificial_selection(*seed).next_action(genealogy, 0)
                == Some(Action::Procreation { progenitor })
        })
        .count()
}

#[test]
fn a_procreation_is_the_next_action_when_the_harness_count_is_within_started_to_the_growth() {
    assert_eq!(
        artificial_selection(0).next_action(&with_a_child_that_attempted(0), 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_harness_with_the_better_clade_is_the_progenitor_more_often_than_one_with_better_results() {
    let genealogy = two_lines();
    assert!(times_progenitor(&genealogy, 1) > times_progenitor(&genealogy, 2));
}

#[test]
fn a_child_that_was_just_created_is_never_the_progenitor() {
    assert_eq!(times_progenitor(&with_a_child_that_attempted(0), 0), 200);
}

#[test]
fn a_harness_that_attempted_every_task_but_one_is_never_the_progenitor() {
    assert_eq!(times_progenitor(&with_a_child_that_attempted(19), 0), 200);
}

#[test]
fn a_harness_given_every_task_is_never_the_progenitor_while_an_attempt_of_it_is_running() {
    let mut genealogy = with_a_child_that_attempted(19);
    genealogy.start_attempt(1, "task-19").unwrap();
    assert_eq!(times_progenitor(&genealogy, 0), 200);
}

#[test]
fn a_child_can_be_the_progenitor_once_it_has_attempted_every_task() {
    assert!(times_progenitor(&with_a_child_that_attempted(20), 1) > 0);
}

#[test]
fn a_harness_that_is_being_created_is_never_the_progenitor() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(times_progenitor(&genealogy, 0), 200);
}

#[test]
fn a_failed_harness_is_never_the_progenitor() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assert_eq!(times_progenitor(&genealogy, 0), 200);
}

#[test]
fn a_procreation_is_the_next_action_even_when_no_task_is_left_to_give() {
    let mut genealogy = Genealogy::new().holding_out(&tasks());
    attempt(&mut genealogy, 0, 20, AttemptResult::Completed);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_harness_that_completed_no_task_can_be_the_progenitor() {
    assert_eq!(
        artificial_selection(0).next_action(&with_the_primogenitor_benchmarked_in_full(), 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_harness_whose_attempts_ran_past_their_time_limit_can_be_the_progenitor() {
    let mut genealogy = Genealogy::new().holding_out(&tasks());
    attempt(&mut genealogy, 0, 20, AttemptResult::TimedOut);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn an_attempt_takes_the_place_of_a_procreation_no_harness_has_attempted_enough_for() {
    let mut genealogy = Genealogy::new().holding_out(&tasks());
    attempt(&mut genealogy, 0, 19, AttemptResult::Completed);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Attempt {
            harness: 0,
            task: "task-19".to_owned()
        })
    );
}
