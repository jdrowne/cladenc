use std::collections::BTreeSet;

use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

const CREATED: CreationOutcome = CreationOutcome::Created;
const FAILED: CreationOutcome = CreationOutcome::Failed;

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_procreations: u64, seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations,
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
        genealogy.start_trial(harness, &task).unwrap();
        genealogy
            .end_trial(harness, &task, TrialResult::NotCompleted)
            .unwrap();
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    benchmark_in_full(&mut genealogy, 0);
    genealogy
}

fn two_harnesses_benchmarked_in_full_after(
    procreations: &[(HarnessId, CreationOutcome)],
) -> Genealogy {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CREATED);
    benchmark_in_full(&mut genealogy, 1);
    for (progenitor, outcome) in procreations {
        child_of(&mut genealogy, *progenitor, *outcome);
    }
    genealogy
}

fn progenitors(genealogy: &Genealogy) -> Vec<HarnessId> {
    let chosen: BTreeSet<HarnessId> = (0..200)
        .filter_map(
            |seed| match artificial_selection(1000, seed).next_action(genealogy, 0) {
                Some(Action::Procreation { progenitor }) => Some(progenitor),
                Some(Action::Trial { .. }) | None => None,
            },
        )
        .collect();
    chosen.into_iter().collect()
}

fn procreations_of_the_primogenitor_alone_when_each_creates_no_child() -> u64 {
    let artificial_selection = artificial_selection(100, 0);
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    let mut action = 0;
    while let Some(Action::Procreation { progenitor }) =
        artificial_selection.next_action(&genealogy, action)
    {
        child_of(&mut genealogy, progenitor, FAILED);
        action += 1;
    }
    genealogy.procreations_started()
}

#[test]
fn when_every_harness_waits_and_no_other_work_is_left_the_one_whose_wait_ends_first_is_the_progenitor()
 {
    let genealogy = two_harnesses_benchmarked_in_full_after(&[
        (1, FAILED),
        (1, FAILED),
        (1, FAILED),
        (0, FAILED),
    ]);
    assert_eq!(progenitors(&genealogy), [0]);
}

#[test]
fn when_the_waits_of_two_harnesses_end_at_the_same_procreation_either_can_be_the_progenitor() {
    let genealogy =
        two_harnesses_benchmarked_in_full_after(&[(1, FAILED), (1, FAILED), (0, FAILED)]);
    assert_eq!(progenitors(&genealogy), [0, 1]);
}

#[test]
fn the_primogenitor_alone_is_the_progenitor_of_every_procreation_though_each_creates_no_child() {
    assert_eq!(
        procreations_of_the_primogenitor_alone_when_each_creates_no_child(),
        100
    );
}

#[test]
fn a_child_whose_procreation_left_the_progenitor_s_copy_unchanged_is_no_work_left_to_do() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CreationOutcome::Unchanged);
    assert_eq!(progenitors(&genealogy), [0]);
}

#[test]
fn a_harness_that_waits_stays_left_out_while_another_harness_has_a_task_not_yet_given() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CREATED);
    child_of(&mut genealogy, 0, FAILED);
    assert_eq!(
        artificial_selection(1000, 0).next_action(&genealogy, 0),
        Some(Action::Trial {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_harness_that_waits_stays_left_out_while_a_trial_of_another_harness_is_still_running() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    child_of(&mut genealogy, 0, CREATED);
    for task in tasks() {
        genealogy.start_trial(1, &task).unwrap();
    }
    child_of(&mut genealogy, 0, FAILED);
    assert_eq!(
        artificial_selection(1000, 0).next_action(&genealogy, 0),
        None
    );
}

#[test]
fn a_harness_that_waits_stays_left_out_while_a_procreation_is_still_running() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_procreation(0).unwrap();
    child_of(&mut genealogy, 0, FAILED);
    assert_eq!(
        artificial_selection(1000, 0).next_action(&genealogy, 0),
        None
    );
}
