use cladenc::genealogy::{ExpansionOutcome, Genealogy, GenealogyError};

fn with_a_child_being_created() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    genealogy
}

fn with_a_failed_child() -> Genealogy {
    let mut genealogy = with_a_child_being_created();
    genealogy
        .end_expansion(1, ExpansionOutcome::Failed)
        .unwrap();
    genealogy
}

#[test]
fn an_expansion_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().start_expansion(7),
        Err(GenealogyError::ParentNotCreated { parent: 7 })
    );
}

#[test]
fn a_refused_expansion_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.start_expansion(7);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn an_expansion_of_a_harness_that_is_being_created_is_refused() {
    assert_eq!(
        with_a_child_being_created().start_expansion(1),
        Err(GenealogyError::ParentNotCreated { parent: 1 })
    );
}

#[test]
fn a_refused_expansion_of_a_harness_being_created_changes_nothing() {
    let mut genealogy = with_a_child_being_created();
    let _ = genealogy.start_expansion(1);
    assert_eq!(genealogy, with_a_child_being_created());
}

#[test]
fn an_expansion_of_a_failed_harness_is_refused() {
    assert_eq!(
        with_a_failed_child().start_expansion(1),
        Err(GenealogyError::ParentNotCreated { parent: 1 })
    );
}

#[test]
fn a_refused_expansion_of_a_failed_harness_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.start_expansion(1);
    assert_eq!(genealogy, with_a_failed_child());
}

#[test]
fn the_end_of_an_expansion_whose_child_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().end_expansion(7, ExpansionOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 7 })
    );
}

#[test]
fn a_refused_end_of_an_expansion_of_a_missing_child_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_expansion(7, ExpansionOutcome::Created);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn the_end_of_an_expansion_of_the_progenitor_is_refused() {
    assert_eq!(
        Genealogy::new().end_expansion(0, ExpansionOutcome::Failed),
        Err(GenealogyError::ChildNotBeingCreated { child: 0 })
    );
}

#[test]
fn a_refused_end_of_an_expansion_of_the_progenitor_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_expansion(0, ExpansionOutcome::Failed);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_second_end_of_an_expansion_is_refused() {
    assert_eq!(
        with_a_failed_child().end_expansion(1, ExpansionOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 1 })
    );
}

#[test]
fn a_refused_second_end_of_an_expansion_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.end_expansion(1, ExpansionOutcome::Created);
    assert_eq!(genealogy, with_a_failed_child());
}
