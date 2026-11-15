use cladenc::genealogy::{ExpansionOutcome, Genealogy, HarnessId};

fn created_ids(genealogy: &Genealogy) -> Vec<HarnessId> {
    genealogy
        .created_harnesses()
        .iter()
        .map(|harness| harness.id())
        .collect()
}

fn expand(genealogy: &mut Genealogy, parent: HarnessId, outcome: ExpansionOutcome) {
    let child = genealogy.start_expansion(parent).unwrap();
    genealogy.end_expansion(child, outcome).unwrap();
}

#[test]
fn the_progenitor_is_the_one_created_harness_of_a_new_genealogy() {
    assert_eq!(created_ids(&Genealogy::new()), [0]);
}

#[test]
fn a_created_child_is_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    expand(&mut genealogy, 0, ExpansionOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 1]);
}

#[test]
fn a_child_being_created_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_failed_harness_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    expand(&mut genealogy, 0, ExpansionOutcome::Failed);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_harness_whose_expansion_timed_out_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    expand(&mut genealogy, 0, ExpansionOutcome::TimedOut);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn the_created_harnesses_come_in_ascending_id_order() {
    let mut genealogy = Genealogy::new();
    expand(&mut genealogy, 0, ExpansionOutcome::Failed);
    expand(&mut genealogy, 0, ExpansionOutcome::Created);
    expand(&mut genealogy, 2, ExpansionOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 2, 3]);
}
