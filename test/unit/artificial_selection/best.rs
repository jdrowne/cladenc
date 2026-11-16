use cladenc::artificial_selection::ArtificialSelection;
use cladenc::genealogy::{EvaluationOutcome, Genealogy, HarnessId, ProcreationOutcome};

fn tasks() -> Vec<String> {
    (0..5).map(|number| format!("task-{number}")).collect()
}

fn best(genealogy: &Genealogy) -> HarnessId {
    let artificial_selection = ArtificialSelection {
        budget: 100,
        growth: 0.6,
        confidence: 99.0,
        tasks: tasks(),
        seed: 0,
    };
    artificial_selection.best_harness(genealogy)
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
    for task in tasks().iter().take(count) {
        genealogy.start_evaluation(harness, task).unwrap();
        genealogy.end_evaluation(harness, task, outcome).unwrap();
    }
}

#[test]
fn the_progenitor_alone_is_the_best_harness() {
    assert_eq!(best(&Genealogy::new()), 0);
}

#[test]
fn the_harness_with_the_largest_lower_bound_is_the_best() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 1, 1, EvaluationOutcome::Success);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_harness_with_a_smaller_lower_bound_does_not_replace_the_best() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 1, 1, EvaluationOutcome::Failure);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn the_lower_id_wins_a_tie_between_harnesses_with_no_evaluations() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn the_lower_id_wins_a_tie_between_harnesses_with_the_same_results() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 1, 2, EvaluationOutcome::Success);
    evaluate(&mut genealogy, 2, 2, EvaluationOutcome::Success);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_failed_harness_is_never_the_best() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    evaluate(&mut genealogy, 0, 1, EvaluationOutcome::Failure);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_harness_that_is_being_created_is_never_the_best() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    evaluate(&mut genealogy, 0, 1, EvaluationOutcome::Failure);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn the_best_harness_is_judged_by_its_own_results_and_not_its_clade() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    evaluate(&mut genealogy, 0, 1, EvaluationOutcome::Success);
    evaluate(&mut genealogy, 1, 3, EvaluationOutcome::Success);
    assert_eq!(best(&genealogy), 1);
}
