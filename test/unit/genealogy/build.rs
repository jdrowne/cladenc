use cladenc::genealogy::{Genealogy, GenealogyError, HarnessState};

fn built() -> Genealogy {
    let mut genealogy = Genealogy::unbuilt();
    genealogy.build_progenitor().unwrap();
    genealogy
}

#[test]
fn a_genealogy_whose_progenitor_is_to_be_built_awaits_the_build() {
    assert!(Genealogy::unbuilt().awaits_build());
}

#[test]
fn a_new_genealogy_awaits_no_build() {
    assert!(!Genealogy::new().awaits_build());
}

#[test]
fn a_progenitor_that_awaits_its_build_is_being_created() {
    assert_eq!(
        Genealogy::unbuilt().harnesses()[0].state(),
        HarnessState::BeingCreated
    );
}

#[test]
fn a_progenitor_that_awaits_its_build_is_not_among_the_created_harnesses() {
    assert!(Genealogy::unbuilt().created_harnesses().is_empty());
}

#[test]
fn a_genealogy_whose_progenitor_was_built_is_a_new_genealogy() {
    assert_eq!(built(), Genealogy::new());
}

#[test]
fn a_genealogy_whose_progenitor_was_built_awaits_no_build() {
    assert!(!built().awaits_build());
}

#[test]
fn building_a_progenitor_that_awaits_no_build_is_refused() {
    assert_eq!(
        Genealogy::new().build_progenitor(),
        Err(GenealogyError::ProgenitorNotAwaitingBuild)
    );
}

#[test]
fn building_a_progenitor_twice_is_refused() {
    assert_eq!(
        built().build_progenitor(),
        Err(GenealogyError::ProgenitorNotAwaitingBuild)
    );
}

#[test]
fn a_progenitor_that_awaits_its_build_cannot_procreate() {
    assert_eq!(
        Genealogy::unbuilt().start_procreation(0),
        Err(GenealogyError::ParentNotCreated { parent: 0 })
    );
}

#[test]
fn a_progenitor_that_awaits_its_build_cannot_be_assessed() {
    assert_eq!(
        Genealogy::unbuilt().start_assessment(0, "t1"),
        Err(GenealogyError::HarnessNotCreated { harness: 0 })
    );
}

#[test]
fn a_progenitor_that_was_built_can_procreate() {
    assert_eq!(built().start_procreation(0), Ok(1));
}
