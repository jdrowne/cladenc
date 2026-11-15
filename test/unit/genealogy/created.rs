use cladenc::genealogy::{Genealogy, HarnessId, MutationOutcome};

fn created_ids(genealogy: &Genealogy) -> Vec<HarnessId> {
    genealogy
        .created_harnesses()
        .iter()
        .map(|harness| harness.id())
        .collect()
}

fn mutate(genealogy: &mut Genealogy, parent: HarnessId, outcome: MutationOutcome) {
    let child = genealogy.start_mutation(parent).unwrap();
    genealogy.end_mutation(child, outcome).unwrap();
}

#[test]
fn the_progenitor_is_the_one_created_harness_of_a_new_genealogy() {
    assert_eq!(created_ids(&Genealogy::new()), [0]);
}

#[test]
fn a_created_child_is_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    mutate(&mut genealogy, 0, MutationOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 1]);
}

#[test]
fn a_child_being_created_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_failed_harness_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    mutate(&mut genealogy, 0, MutationOutcome::Failed);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn a_harness_whose_mutation_timed_out_is_not_among_the_created_harnesses() {
    let mut genealogy = Genealogy::new();
    mutate(&mut genealogy, 0, MutationOutcome::TimedOut);
    assert_eq!(created_ids(&genealogy), [0]);
}

#[test]
fn the_created_harnesses_come_in_ascending_id_order() {
    let mut genealogy = Genealogy::new();
    mutate(&mut genealogy, 0, MutationOutcome::Failed);
    mutate(&mut genealogy, 0, MutationOutcome::Created);
    mutate(&mut genealogy, 2, MutationOutcome::Created);
    assert_eq!(created_ids(&genealogy), [0, 2, 3]);
}
