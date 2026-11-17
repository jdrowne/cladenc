use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_procreations: u64, growth: f64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations,
        growth,
        tasks: tasks(),
        seed: 0,
    }
}

fn benchmark_in_full(genealogy: &mut Genealogy, harness: HarnessId) {
    for task in tasks() {
        genealogy.start_attempt(harness, &task).unwrap();
        genealogy
            .end_attempt(harness, &task, AttemptResult::NotCompleted)
            .unwrap();
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new();
    benchmark_in_full(&mut genealogy, 0);
    genealogy
}

fn with_a_child(outcome: ProcreationOutcome) -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
    genealogy
}

fn is_a_procreation(action: Option<Action>) -> bool {
    matches!(action, Some(Action::Procreation { .. }))
}

#[test]
fn a_procreation_the_growth_does_not_let_start_is_due_once_every_created_harness_is_benchmarked_in_full()
 {
    let mut genealogy = with_a_child(ProcreationOutcome::Created);
    benchmark_in_full(&mut genealogy, 1);
    assert!(is_a_procreation(
        artificial_selection(100, 0.0).next_action(&genealogy, 0)
    ));
}

#[test]
fn a_failed_harness_does_not_hold_back_a_procreation_that_is_due_for_lack_of_other_work() {
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&with_a_child(ProcreationOutcome::Failed), 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_created_harness_with_a_task_not_yet_given_attempts_it_before_such_a_procreation() {
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&with_a_child(ProcreationOutcome::Created), 0),
        Some(Action::Attempt {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn no_procreation_is_due_for_lack_of_other_work_while_an_attempt_is_running() {
    let mut genealogy = with_a_child(ProcreationOutcome::Created);
    genealogy.start_attempt(1, "task-0").unwrap();
    genealogy.start_attempt(1, "task-1").unwrap();
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&genealogy, 0),
        None
    );
}

#[test]
fn a_procreation_the_growth_lets_start_is_not_the_next_action_once_the_most_have_started() {
    assert_eq!(
        artificial_selection(1, 1.0).next_action(&with_a_child(ProcreationOutcome::Created), 0),
        Some(Action::Attempt {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_procreation_is_still_the_next_action_when_one_fewer_than_the_most_have_started() {
    assert_eq!(
        artificial_selection(2, 1.0).next_action(&with_a_child(ProcreationOutcome::Created), 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_procreation_that_failed_counts_among_those_started() {
    assert_eq!(
        artificial_selection(1, 1.0).next_action(&with_a_child(ProcreationOutcome::Failed), 0),
        None
    );
}

#[test]
fn no_procreation_is_due_for_lack_of_other_work_once_the_most_have_started() {
    let mut genealogy = with_a_child(ProcreationOutcome::Created);
    benchmark_in_full(&mut genealogy, 1);
    assert_eq!(
        artificial_selection(1, 0.0).next_action(&genealogy, 0),
        None
    );
}
