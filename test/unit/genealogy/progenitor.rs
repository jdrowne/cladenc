use cladenc::genealogy::{Genealogy, HarnessState};

#[test]
fn a_new_genealogy_holds_one_harness() {
    assert_eq!(Genealogy::new().harness_count(), 1);
}

#[test]
fn the_progenitor_is_harness_0() {
    assert_eq!(Genealogy::new().harnesses()[0].id(), 0);
}

#[test]
fn the_progenitor_is_created_from_the_start() {
    assert_eq!(
        Genealogy::new().harnesses()[0].state(),
        HarnessState::Created
    );
}

#[test]
fn the_progenitor_has_no_parent() {
    assert_eq!(Genealogy::new().harnesses()[0].parent(), None);
}

#[test]
fn a_new_genealogy_has_started_no_evaluation() {
    assert_eq!(Genealogy::new().started(), 0);
}

#[test]
fn the_progenitor_starts_with_no_successes() {
    assert_eq!(Genealogy::new().harnesses()[0].successes(), 0);
}

#[test]
fn the_progenitor_starts_with_no_failures() {
    assert_eq!(Genealogy::new().harnesses()[0].failures(), 0);
}

#[test]
fn the_default_genealogy_is_a_new_genealogy() {
    assert_eq!(Genealogy::default(), Genealogy::new());
}
