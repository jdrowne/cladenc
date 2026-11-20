use crate::genealogy::{CreationOutcome, Genealogy, HarnessState};

fn procreated(outcome: CreationOutcome) -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
    genealogy
}

#[test]
fn the_first_child_takes_id_1() {
    assert_eq!(Genealogy::new().start_procreation(0), Ok(1));
}

#[test]
fn the_second_child_takes_id_2() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.start_procreation(0), Ok(2));
}

#[test]
fn a_child_takes_the_next_id_after_a_failed_child() {
    let mut genealogy = procreated(CreationOutcome::Failed);
    assert_eq!(genealogy.start_procreation(0), Ok(2));
}

#[test]
fn a_child_carries_its_own_id() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].id(), 1);
}

#[test]
fn a_child_is_joined_to_its_progenitor() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].progenitor(), Some(0));
}

#[test]
fn a_child_of_a_child_is_joined_to_that_child() {
    let mut genealogy = procreated(CreationOutcome::Created);
    genealogy.start_procreation(1).unwrap();
    assert_eq!(genealogy.harnesses()[2].progenitor(), Some(1));
}

#[test]
fn a_child_is_being_created_while_its_procreation_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::BeingCreated);
}

#[test]
fn a_procreation_that_created_its_child_leaves_a_created_harness() {
    let genealogy = procreated(CreationOutcome::Created);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Created);
}

#[test]
fn a_procreation_that_failed_leaves_a_failed_harness() {
    let genealogy = procreated(CreationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn a_procreation_whose_child_did_not_build_leaves_a_failed_harness() {
    let genealogy = procreated(CreationOutcome::BuildFailed);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn a_procreation_that_timed_out_leaves_a_failed_harness() {
    let genealogy = procreated(CreationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn harness_count_counts_a_child_that_is_being_created() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_was_created() {
    assert_eq!(procreated(CreationOutcome::Created).harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_failed() {
    assert_eq!(procreated(CreationOutcome::Failed).harness_count(), 2);
}

#[test]
fn a_new_genealogy_has_started_no_procreation() {
    assert_eq!(Genealogy::new().procreations_started(), 0);
}

#[test]
fn a_procreation_counts_as_started_while_it_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(genealogy.procreations_started(), 1);
}

#[test]
fn a_procreation_that_failed_still_counts_as_started() {
    let genealogy = procreated(CreationOutcome::Failed);
    assert_eq!(genealogy.procreations_started(), 1);
}
