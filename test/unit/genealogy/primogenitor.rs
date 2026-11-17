use cladenc::genealogy::{Genealogy, HarnessState};

#[test]
fn a_new_genealogy_holds_one_harness() {
    assert_eq!(Genealogy::new().harness_count(), 1);
}

#[test]
fn the_primogenitor_is_harness_0() {
    assert_eq!(Genealogy::new().harnesses()[0].id(), 0);
}

#[test]
fn the_primogenitor_is_created_from_the_start() {
    assert_eq!(
        Genealogy::new().harnesses()[0].state(),
        HarnessState::Created
    );
}

#[test]
fn the_primogenitor_has_no_progenitor() {
    assert_eq!(Genealogy::new().harnesses()[0].progenitor(), None);
}

#[test]
fn a_new_genealogy_has_started_no_attempt() {
    assert_eq!(Genealogy::new().started(), 0);
}

#[test]
fn the_primogenitor_starts_with_nothing_completed() {
    assert_eq!(Genealogy::new().harnesses()[0].completed(), 0);
}

#[test]
fn the_primogenitor_starts_with_nothing_not_completed() {
    assert_eq!(Genealogy::new().harnesses()[0].not_completed(), 0);
}

#[test]
fn the_default_genealogy_is_a_new_genealogy() {
    assert_eq!(Genealogy::default(), Genealogy::new());
}
