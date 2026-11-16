use cladenc::genealogy::{AssessmentOutcome, Genealogy, GenealogyError, ProcreationOutcome};

fn with_a_child_being_created() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    genealogy
}

fn with_a_failed_child() -> Genealogy {
    let mut genealogy = with_a_child_being_created();
    genealogy
        .end_procreation(1, ProcreationOutcome::Failed)
        .unwrap();
    genealogy
}

fn with_an_assessment_running() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    genealogy
}

fn with_an_assessment_ended() -> Genealogy {
    let mut genealogy = with_an_assessment_running();
    genealogy
        .end_assessment(0, "a", AssessmentOutcome::Success)
        .unwrap();
    genealogy
}

fn not_running(harness: u64, task: &str) -> GenealogyError {
    GenealogyError::AssessmentNotRunning {
        harness,
        task: task.to_owned(),
    }
}

fn already_given(harness: u64, task: &str) -> GenealogyError {
    GenealogyError::TaskAlreadyGiven {
        harness,
        task: task.to_owned(),
    }
}

#[test]
fn an_assessment_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().start_assessment(7, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 7 })
    );
}

#[test]
fn a_refused_assessment_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.start_assessment(7, "a");
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn an_assessment_of_a_harness_that_is_being_created_is_refused() {
    assert_eq!(
        with_a_child_being_created().start_assessment(1, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 1 })
    );
}

#[test]
fn a_refused_assessment_of_a_harness_being_created_changes_nothing() {
    let mut genealogy = with_a_child_being_created();
    let _ = genealogy.start_assessment(1, "a");
    assert_eq!(genealogy, with_a_child_being_created());
}

#[test]
fn an_assessment_of_a_failed_harness_is_refused() {
    assert_eq!(
        with_a_failed_child().start_assessment(1, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 1 })
    );
}

#[test]
fn a_refused_assessment_of_a_failed_harness_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.start_assessment(1, "a");
    assert_eq!(genealogy, with_a_failed_child());
}

#[test]
fn a_second_assessment_on_a_task_whose_assessment_runs_is_refused() {
    assert_eq!(
        with_an_assessment_running().start_assessment(0, "a"),
        Err(already_given(0, "a"))
    );
}

#[test]
fn a_refused_second_assessment_on_a_running_task_changes_nothing() {
    let mut genealogy = with_an_assessment_running();
    let _ = genealogy.start_assessment(0, "a");
    assert_eq!(genealogy, with_an_assessment_running());
}

#[test]
fn a_second_assessment_on_a_task_whose_assessment_ended_is_refused() {
    assert_eq!(
        with_an_assessment_ended().start_assessment(0, "a"),
        Err(already_given(0, "a"))
    );
}

#[test]
fn a_refused_second_assessment_on_an_ended_task_changes_nothing() {
    let mut genealogy = with_an_assessment_ended();
    let _ = genealogy.start_assessment(0, "a");
    assert_eq!(genealogy, with_an_assessment_ended());
}

#[test]
fn the_end_of_an_assessment_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().end_assessment(7, "a", AssessmentOutcome::Success),
        Err(not_running(7, "a"))
    );
}

#[test]
fn a_refused_end_of_an_assessment_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_assessment(7, "a", AssessmentOutcome::Success);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn the_end_of_an_assessment_on_a_task_never_given_is_refused() {
    assert_eq!(
        with_an_assessment_running().end_assessment(0, "b", AssessmentOutcome::Success),
        Err(not_running(0, "b"))
    );
}

#[test]
fn a_refused_end_of_an_assessment_on_a_task_never_given_changes_nothing() {
    let mut genealogy = with_an_assessment_running();
    let _ = genealogy.end_assessment(0, "b", AssessmentOutcome::Success);
    assert_eq!(genealogy, with_an_assessment_running());
}

#[test]
fn a_second_end_of_an_assessment_is_refused() {
    assert_eq!(
        with_an_assessment_ended().end_assessment(0, "a", AssessmentOutcome::Failure),
        Err(not_running(0, "a"))
    );
}

#[test]
fn a_refused_second_end_of_an_assessment_changes_nothing() {
    let mut genealogy = with_an_assessment_ended();
    let _ = genealogy.end_assessment(0, "a", AssessmentOutcome::Failure);
    assert_eq!(genealogy, with_an_assessment_ended());
}
