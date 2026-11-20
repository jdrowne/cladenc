use std::collections::BTreeSet;

use crate::artificial_selection::wait::Waits;
use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy, HarnessId};

const CREATED: CreationOutcome = CreationOutcome::Created;
const FAILED: CreationOutcome = CreationOutcome::Failed;

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations: 1000,
        tasks: tasks(),
        seed,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn benchmark_in_full(genealogy: &mut Genealogy, harness: HarnessId) {
    for task in tasks() {
        genealogy.start_attempt(harness, &task).unwrap();
        genealogy
            .end_attempt(harness, &task, AttemptResult::NotCompleted)
            .unwrap();
    }
}

fn two_harnesses_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    benchmark_in_full(&mut genealogy, 0);
    child_of(&mut genealogy, 0, CREATED);
    benchmark_in_full(&mut genealogy, 1);
    genealogy
}

fn after(procreations: &[(HarnessId, CreationOutcome)]) -> Genealogy {
    let mut genealogy = two_harnesses_benchmarked_in_full();
    for (progenitor, outcome) in procreations {
        child_of(&mut genealogy, *progenitor, *outcome);
    }
    genealogy
}

fn progenitors(genealogy: &Genealogy) -> Vec<HarnessId> {
    let chosen: BTreeSet<HarnessId> = (0..200)
        .filter_map(
            |seed| match artificial_selection(seed).next_action(genealogy, 0) {
                Some(Action::Procreation { progenitor }) => Some(progenitor),
                Some(Action::Attempt { .. }) | None => None,
            },
        )
        .collect();
    chosen.into_iter().collect()
}

fn is_left_out(genealogy: &Genealogy) -> bool {
    Waits::of(genealogy).has_one_for(&genealogy.harnesses()[1])
}

fn procreations_of_a_harness_that_never_creates_a_child() -> Vec<u64> {
    let mut genealogy = Genealogy::new();
    child_of(&mut genealogy, 0, CREATED);
    let mut numbers = Vec::new();
    while genealogy.procreations_started() < 100 {
        let number = genealogy.harness_count();
        if Waits::of(&genealogy).has_one_for(&genealogy.harnesses()[1]) {
            child_of(&mut genealogy, 0, CREATED);
        } else {
            child_of(&mut genealogy, 1, FAILED);
            numbers.push(number);
        }
    }
    numbers
}

#[test]
fn a_harness_that_made_no_child_yet_can_be_the_progenitor() {
    assert_eq!(progenitors(&after(&[])), [0, 1]);
}

#[test]
fn a_harness_is_left_out_of_the_procreation_after_one_of_its_own_that_failed() {
    assert_eq!(progenitors(&after(&[(1, FAILED)])), [0]);
}

#[test]
fn a_harness_is_left_out_of_the_procreation_after_one_of_its_own_whose_child_did_not_build() {
    assert_eq!(
        progenitors(&after(&[(1, CreationOutcome::BuildFailed)])),
        [0]
    );
}

#[test]
fn a_harness_is_left_out_of_the_procreation_after_one_of_its_own_that_ran_past_its_time_limit() {
    assert_eq!(progenitors(&after(&[(1, CreationOutcome::TimedOut)])), [0]);
}

#[test]
fn a_harness_is_left_out_of_the_procreation_after_one_of_its_own_that_left_its_copy_unchanged() {
    assert_eq!(progenitors(&after(&[(1, CreationOutcome::Unchanged)])), [0]);
}

#[test]
fn a_harness_can_be_the_progenitor_again_one_procreation_after_its_first_that_created_no_child() {
    assert!(!is_left_out(&after(&[(1, FAILED), (0, CREATED)])));
}

#[test]
fn a_harness_is_left_out_of_the_first_procreation_after_its_second_in_a_row_that_created_no_child()
{
    assert!(is_left_out(&after(&[
        (1, FAILED),
        (0, CREATED),
        (1, FAILED)
    ])));
}

#[test]
fn a_harness_is_left_out_of_the_second_procreation_after_its_second_in_a_row_that_created_no_child()
{
    assert!(is_left_out(&after(&[
        (1, FAILED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED)
    ])));
}

#[test]
fn a_harness_can_be_the_progenitor_again_two_procreations_after_its_second_in_a_row_that_created_no_child()
 {
    assert!(!is_left_out(&after(&[
        (1, FAILED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED),
        (0, CREATED)
    ])));
}

#[test]
fn a_harness_is_left_out_of_the_fourth_procreation_after_its_third_in_a_row_that_created_no_child()
{
    assert!(is_left_out(&after(&[
        (1, FAILED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED),
        (0, CREATED),
        (0, CREATED)
    ])));
}

#[test]
fn a_harness_can_be_the_progenitor_again_four_procreations_after_its_third_in_a_row_that_created_no_child()
 {
    assert!(!is_left_out(&after(&[
        (1, FAILED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED),
        (0, CREATED),
        (1, FAILED),
        (0, CREATED),
        (0, CREATED),
        (0, CREATED),
        (0, CREATED)
    ])));
}

#[test]
fn a_procreation_that_created_a_child_ends_the_wait_of_its_progenitor() {
    assert!(!is_left_out(&after(&[
        (1, FAILED),
        (1, FAILED),
        (1, CREATED)
    ])));
}

#[test]
fn a_procreation_that_creates_no_child_after_one_that_created_a_child_is_counted_as_the_first() {
    assert!(!is_left_out(&after(&[
        (1, FAILED),
        (1, FAILED),
        (1, CREATED),
        (1, FAILED),
        (0, CREATED)
    ])));
}

#[test]
fn a_procreation_that_is_still_running_does_not_lengthen_the_wait_of_its_progenitor() {
    let mut genealogy = after(&[(1, FAILED), (0, CREATED)]);
    genealogy.start_procreation(1).unwrap();
    assert!(!is_left_out(&genealogy));
}

#[test]
fn a_procreation_that_is_still_running_does_not_end_the_wait_of_its_progenitor() {
    let mut genealogy = after(&[(1, FAILED)]);
    genealogy.start_procreation(1).unwrap();
    child_of(&mut genealogy, 1, FAILED);
    child_of(&mut genealogy, 0, CREATED);
    assert!(is_left_out(&genealogy));
}

#[test]
fn a_harness_that_waits_can_be_the_progenitor_again_once_its_wait_is_over_and_no_attempt_is_left() {
    let mut genealogy = after(&[(1, FAILED), (0, CREATED)]);
    benchmark_in_full(&mut genealogy, 3);
    assert!(progenitors(&genealogy).contains(&1));
}

#[test]
fn a_harness_that_never_creates_a_child_and_procreates_whenever_it_does_not_wait_takes_7_of_100_procreations()
 {
    assert_eq!(
        procreations_of_a_harness_that_never_creates_a_child(),
        [2, 4, 7, 12, 21, 38, 71]
    );
}
