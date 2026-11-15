use cladenc::genealogy::{ExpansionOutcome, Genealogy, HarnessState};

fn expanded(outcome: ExpansionOutcome) -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_expansion(0).unwrap();
    genealogy.end_expansion(child, outcome).unwrap();
    genealogy
}

#[test]
fn the_first_child_takes_id_1() {
    assert_eq!(Genealogy::new().start_expansion(0), Ok(1));
}

#[test]
fn the_second_child_takes_id_2() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(genealogy.start_expansion(0), Ok(2));
}

#[test]
fn a_child_takes_the_next_id_after_a_failed_child() {
    let mut genealogy = expanded(ExpansionOutcome::Failed);
    assert_eq!(genealogy.start_expansion(0), Ok(2));
}

#[test]
fn a_child_carries_its_own_id() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].id(), 1);
}

#[test]
fn a_child_is_joined_to_its_parent() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].parent(), Some(0));
}

#[test]
fn a_child_of_a_child_is_joined_to_that_child() {
    let mut genealogy = expanded(ExpansionOutcome::Created);
    genealogy.start_expansion(1).unwrap();
    assert_eq!(genealogy.harnesses()[2].parent(), Some(1));
}

#[test]
fn a_child_is_being_created_while_its_expansion_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::BeingCreated);
}

#[test]
fn an_expansion_that_created_its_child_leaves_a_created_harness() {
    let genealogy = expanded(ExpansionOutcome::Created);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Created);
}

#[test]
fn an_expansion_that_failed_leaves_a_failed_harness() {
    let genealogy = expanded(ExpansionOutcome::Failed);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn an_expansion_that_timed_out_leaves_a_failed_harness() {
    let genealogy = expanded(ExpansionOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[1].state(), HarnessState::Failed);
}

#[test]
fn harness_count_counts_a_child_that_is_being_created() {
    let mut genealogy = Genealogy::new();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(genealogy.harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_was_created() {
    assert_eq!(expanded(ExpansionOutcome::Created).harness_count(), 2);
}

#[test]
fn harness_count_counts_a_child_that_failed() {
    assert_eq!(expanded(ExpansionOutcome::Failed).harness_count(), 2);
}
