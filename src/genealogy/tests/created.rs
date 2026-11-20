use crate::genealogy::{CreationOutcome, Genealogy, HarnessId};

fn created_ids(genealogy: &Genealogy) -> Vec<HarnessId> {
    genealogy
        .created_harnesses()
        .iter()
        .map(|harness| harness.id())
        .collect()
}

fn procreate(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

#[test]
fn the_primogenitor_is_the_one_created_harness_of_a_new_genealogy() {
    assert_eq!(created_ids(&Genealogy::new()), [0]);
}

#[test]
fn a_created_child_is_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    procreate(&mut genealogy, 0, CreationOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 1]);
}

#[test]
fn a_child_being_created_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_failed_harness_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    procreate(&mut genealogy, 0, CreationOutcome::Failed);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_harness_whose_procreation_timed_out_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    procreate(&mut genealogy, 0, CreationOutcome::TimedOut);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_harness_whose_procreation_left_the_progenitor_s_copy_unchanged_is_not_among_the_created_harnesses()
 {
    let mut genealogy = Genealogy::new();
    procreate(&mut genealogy, 0, CreationOutcome::Unchanged);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn the_created_harnesses_come_in_ascending_id_order() {
    let mut genealogy = Genealogy::new();
    procreate(&mut genealogy, 0, CreationOutcome::Failed);
    procreate(&mut genealogy, 0, CreationOutcome::Created);
    procreate(&mut genealogy, 2, CreationOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 2, 3]);
}
