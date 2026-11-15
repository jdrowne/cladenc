use cladenc::genealogy::{Genealogy, HarnessState, MutationOutcome};

fn mutated(outcome: MutationOutcome) -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_mutation(0).unwrap();
    genealogy.end_mutation(child, outcome).unwrap();
    genealogy
}

#[test]
fn the_first_child_takes_id_1() {
    assert_eq!(Genealogy::new().start_mutation(0), Ok(1));
}

#[test]
fn the_second_child_takes_id_2() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(genealogy.start_mutation(0), Ok(2));
}

#[test]
fn a_child_takes_the_next_id_after_a_failed_child() {
    let mut genealogy = mutated(MutationOutcome::Failed);
    assert_eq!(genealogy.start_mutation(0), Ok(2));
}

#[test]
fn a_child_carries_its_own_id() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].id(), 1);
}

#[test]
fn a_child_is_joined_to_its_parent() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].parent(), Some(0));
}

#[test]
fn a_child_of_a_child_is_joined_to_that_child() {
    let mut genealogy = mutated(MutationOutcome::Created);
    genealogy.start_mutation(1).unwrap();
    assert_eq!(genealogy.harnesses()[2].parent(), Some(1));
}

#[test]
fn a_child_is_being_created_while_its_mutation_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::BeingCreated);
}

#[test]
fn a_mutation_that_created_its_child_leaves_a_created_harness() {
    let genealogy = mutated(MutationOutcome::Created);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Created);
}

#[test]
fn a_mutation_that_failed_leaves_a_failed_harness() {
    let genealogy = mutated(MutationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn a_mutation_that_timed_out_leaves_a_failed_harness() {
    let genealogy = mutated(MutationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn harness_count_counts_a_child_that_is_being_created() {
    let mut genealogy = Genealogy::new();
    genealogy.start_mutation(0).unwrap();
    assert_eq!(genealogy.harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_was_created() {
    assert_eq!(mutated(MutationOutcome::Created).harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_failed() {
    assert_eq!(mutated(MutationOutcome::Failed).harness_count(), 2);
}
