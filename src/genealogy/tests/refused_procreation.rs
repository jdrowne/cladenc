use crate::genealogy::{Genealogy, GenealogyError, ProcreationOutcome};

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

#[test]
fn a_procreation_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().start_procreation(7),
        Err(GenealogyError::ProgenitorNotCreated { progenitor: 7 })
    );
}

#[test]
fn a_refused_procreation_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.start_procreation(7);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_procreation_of_a_harness_that_is_being_created_is_refused() {
    assert_eq!(
        with_a_child_being_created().start_procreation(1),
        Err(GenealogyError::ProgenitorNotCreated { progenitor: 1 })
    );
}

#[test]
fn a_refused_procreation_of_a_harness_being_created_changes_nothing() {
    let mut genealogy = with_a_child_being_created();
    let _ = genealogy.start_procreation(1);
    assert_eq!(genealogy, with_a_child_being_created());
}

#[test]
fn a_procreation_of_a_failed_harness_is_refused() {
    assert_eq!(
        with_a_failed_child().start_procreation(1),
        Err(GenealogyError::ProgenitorNotCreated { progenitor: 1 })
    );
}

#[test]
fn a_refused_procreation_of_a_failed_harness_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.start_procreation(1);
    assert_eq!(genealogy, with_a_failed_child());
}

#[test]
fn the_end_of_a_procreation_whose_child_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().end_procreation(7, ProcreationOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 7 })
    );
}

#[test]
fn a_refused_end_of_a_procreation_of_a_missing_child_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_procreation(7, ProcreationOutcome::Created);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn the_end_of_a_procreation_of_the_primogenitor_is_refused() {
    assert_eq!(
        Genealogy::new().end_procreation(0, ProcreationOutcome::Failed),
        Err(GenealogyError::ChildNotBeingCreated { child: 0 })
    );
}

#[test]
fn a_refused_end_of_a_procreation_of_the_primogenitor_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_procreation(0, ProcreationOutcome::Failed);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_second_end_of_a_procreation_is_refused() {
    assert_eq!(
        with_a_failed_child().end_procreation(1, ProcreationOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 1 })
    );
}

#[test]
fn a_refused_second_end_of_a_procreation_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.end_procreation(1, ProcreationOutcome::Created);
    assert_eq!(genealogy, with_a_failed_child());
}
