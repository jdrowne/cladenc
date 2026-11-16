use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{AssessmentOutcome, Genealogy, HarnessId, ProcreationOutcome};

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        budget: 1000,
        growth: 1.0,
        confidence: 99.0,
        tasks: tasks(),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, parent: HarnessId, outcome: ProcreationOutcome) {
    let child = genealogy.start_procreation(parent).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn assess(genealogy: &mut Genealogy, harness: HarnessId, count: usize, outcome: AssessmentOutcome) {
    for task in tasks().iter().take(count) {
        genealogy.start_assessment(harness, task).unwrap();
        genealogy.end_assessment(harness, task, outcome).unwrap();
    }
}

fn two_lines() -> Genealogy {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    child_of(&mut genealogy, 1, ProcreationOutcome::Created);
    child_of(&mut genealogy, 2, ProcreationOutcome::Created);
    assess(&mut genealogy, 1, 4, AssessmentOutcome::Failure);
    assess(&mut genealogy, 3, 8, AssessmentOutcome::Success);
    assess(&mut genealogy, 2, 4, AssessmentOutcome::Success);
    assess(&mut genealogy, 4, 8, AssessmentOutcome::Failure);
    genealogy
}

fn times_parent(genealogy: &Genealogy, parent: HarnessId) -> usize {
    (0..200)
        .filter(|seed| {
            artificial_selection(*seed).next_action(genealogy, 0)
                == Some(Action::Procreation { parent })
        })
        .count()
}

#[test]
fn a_procreation_is_the_next_action_when_the_harness_count_is_within_started_to_the_growth() {
    let mut genealogy = Genealogy::new();
    assess(&mut genealogy, 0, 1, AssessmentOutcome::Success);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Procreation { parent: 0 })
    );
}

#[test]
fn a_harness_with_the_better_clade_is_the_parent_more_often_than_one_with_better_results() {
    let genealogy = two_lines();
    assert!(times_parent(&genealogy, 1) > times_parent(&genealogy, 2));
}

#[test]
fn a_created_child_can_be_the_parent() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Created);
    genealogy.start_assessment(0, "task-0").unwrap();
    genealogy.start_assessment(0, "task-1").unwrap();
    assert!(times_parent(&genealogy, 1) > 0);
}

#[test]
fn a_harness_that_is_being_created_is_never_the_parent() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assess(&mut genealogy, 0, 2, AssessmentOutcome::Failure);
    assert_eq!(times_parent(&genealogy, 0), 200);
}

#[test]
fn a_failed_harness_is_never_the_parent() {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, ProcreationOutcome::Failed);
    assess(&mut genealogy, 0, 2, AssessmentOutcome::Failure);
    assert_eq!(times_parent(&genealogy, 0), 200);
}

#[test]
fn a_procreation_is_the_next_action_even_when_no_task_is_left_to_give() {
    let mut genealogy = Genealogy::new();
    assess(&mut genealogy, 0, 20, AssessmentOutcome::Success);
    assert_eq!(
        artificial_selection(0).next_action(&genealogy, 0),
        Some(Action::Procreation { parent: 0 })
    );
}
