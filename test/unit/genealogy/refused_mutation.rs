use cladenc::genealogy::{Genealogy, GenealogyError, MutationOutcome};

fn with_a_child_being_created() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    genealogy
}

fn with_a_failed_child() -> Genealogy {
    let mut genealogy = with_a_child_being_created();
    genealogy.end_mutation(1, MutationOutcome::Failed).unwrap();
    genealogy
}

#[test]
fn a_mutation_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().start_mutation(7),
        Err(GenealogyError::ParentNotCreated { parent: 7 })
    );
}

#[test]
fn a_refused_mutation_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.start_mutation(7);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_mutation_of_a_harness_that_is_being_created_is_refused() {
    assert_eq!(
        with_a_child_being_created().start_mutation(1),
        Err(GenealogyError::ParentNotCreated { parent: 1 })
    );
}

#[test]
fn a_refused_mutation_of_a_harness_being_created_changes_nothing() {
    let mut genealogy = with_a_child_being_created();
    let _ = genealogy.start_mutation(1);
    assert_eq!(genealogy, with_a_child_being_created());
}

#[test]
fn a_mutation_of_a_failed_harness_is_refused() {
    assert_eq!(
        with_a_failed_child().start_mutation(1),
        Err(GenealogyError::ParentNotCreated { parent: 1 })
    );
}

#[test]
fn a_refused_mutation_of_a_failed_harness_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.start_mutation(1);
    assert_eq!(genealogy, with_a_failed_child());
}

#[test]
fn the_end_of_a_mutation_whose_child_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().end_mutation(7, MutationOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 7 })
    );
}

#[test]
fn a_refused_end_of_a_mutation_of_a_missing_child_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_mutation(7, MutationOutcome::Created);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn the_end_of_a_mutation_of_the_progenitor_is_refused() {
    assert_eq!(
        Genealogy::new().end_mutation(0, MutationOutcome::Failed),
        Err(GenealogyError::ChildNotBeingCreated { child: 0 })
    );
}

#[test]
fn a_refused_end_of_a_mutation_of_the_progenitor_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_mutation(0, MutationOutcome::Failed);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_second_end_of_a_mutation_is_refused() {
    assert_eq!(
        with_a_failed_child().end_mutation(1, MutationOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 1 })
    );
}

#[test]
fn a_refused_second_end_of_a_mutation_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.end_mutation(1, MutationOutcome::Created);
    assert_eq!(genealogy, with_a_failed_child());
}
