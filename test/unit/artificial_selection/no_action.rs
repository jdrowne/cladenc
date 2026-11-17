use cladenc::artificial_selection::ArtificialSelection;
use cladenc::genealogy::{Genealogy, ProcreationOutcome, Score};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(budget: u64, growth: f64) -> ArtificialSelection {
    ArtificialSelection {
        budget,
        growth,
        confidence: 99.0,
        tasks: tasks(),
        seed: 0,
    }
}

fn with_every_task_given() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Failed)
        .unwrap();
    for task in tasks() {
        genealogy.start_assessment(0, &task).unwrap();
        genealogy.end_assessment(0, &task, Score::Success).unwrap();
    }
    genealogy
}

#[test]
fn there_is_no_next_action_when_no_procreation_is_due_and_every_task_has_been_given() {
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&with_every_task_given(), 0),
        None
    );
}

#[test]
fn there_is_no_next_action_once_the_budget_is_spent() {
    assert_eq!(
        artificial_selection(2, 1.0).next_action(&with_every_task_given(), 0),
        None
    );
}

#[test]
fn there_is_no_next_action_while_the_only_harness_that_could_procreate_is_still_assessed() {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "task-0").unwrap();
    genealogy.start_assessment(0, "task-1").unwrap();
    assert_eq!(
        artificial_selection(100, 0.0).next_action(&genealogy, 0),
        None
    );
}
