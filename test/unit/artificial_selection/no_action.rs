use cladenc::artificial_selection::ArtificialSelection;
use cladenc::genealogy::{AttemptResult, Genealogy};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_procreations: u64, growth: f64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations,
        growth,
        confidence: 99.0,
        tasks: tasks(),
        seed: 0,
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new();
    for task in tasks() {
        genealogy.start_attempt(0, &task).unwrap();
        genealogy
            .end_attempt(0, &task, AttemptResult::Completed)
            .unwrap();
    }
    genealogy
}

#[test]
fn there_is_no_next_action_while_the_only_harness_that_could_procreate_still_attempts_a_task() {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "task-0").unwrap();
    genealogy.start_attempt(0, "task-1").unwrap();
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&genealogy, 0),
        None
    );
}

#[test]
fn there_is_no_next_action_while_the_last_procreation_is_under_way_and_no_task_is_left_to_give() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(
        artificial_selection(1, 1.0).next_action(&genealogy, 0),
        None
    );
}

#[test]
fn there_is_no_next_action_while_a_procreation_is_under_way_and_the_growth_lets_no_other_start() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&genealogy, 0),
        None
    );
}
