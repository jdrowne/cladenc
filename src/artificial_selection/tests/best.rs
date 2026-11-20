use crate::artificial_selection::ArtificialSelection;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy, HarnessId};

fn tasks() -> Vec<String> {
    (0..5).map(|number| format!("task-{number}")).collect()
}

fn best(genealogy: &Genealogy) -> HarnessId {
    let artificial_selection = ArtificialSelection {
        max_procreations: 100,
        growth: 0.6,
        tasks: tasks(),
        seed: 0,
    };
    artificial_selection.best_harness(genealogy)
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn attempt(genealogy: &mut Genealogy, harness: HarnessId, count: usize, completed: usize) {
    for (number, task) in tasks().iter().take(count).enumerate() {
        let result = if number < completed {
            AttemptResult::Completed
        } else {
            AttemptResult::NotCompleted
        };
        genealogy.start_attempt(harness, task).unwrap();
        genealogy.end_attempt(harness, task, result).unwrap();
    }
}

fn benchmarked_in_full(genealogy: &mut Genealogy, harness: HarnessId, completed: usize) {
    attempt(genealogy, harness, 5, completed);
}

#[test]
fn the_primogenitor_alone_is_the_best_harness() {
    assert_eq!(best(&Genealogy::new().selecting_by(&tasks())), 0);
}

#[test]
fn the_harness_benchmarked_in_full_that_completed_the_most_tasks_is_the_best() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 2);
    benchmarked_in_full(&mut genealogy, 1, 3);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_harness_that_completed_fewer_tasks_does_not_replace_the_best() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 3);
    benchmarked_in_full(&mut genealogy, 1, 2);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn the_lower_id_wins_a_tie_between_harnesses_that_completed_as_many_tasks() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 1);
    benchmarked_in_full(&mut genealogy, 1, 3);
    benchmarked_in_full(&mut genealogy, 2, 3);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn the_lower_id_wins_a_tie_between_harnesses_that_completed_no_task() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 0);
    benchmarked_in_full(&mut genealogy, 1, 0);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_harness_with_a_task_left_is_not_the_best_while_another_is_benchmarked_in_full() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 1);
    attempt(&mut genealogy, 1, 4, 4);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_harness_whose_last_attempt_has_not_ended_is_not_yet_benchmarked_in_full() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 1);
    attempt(&mut genealogy, 1, 4, 4);
    genealogy.start_attempt(1, "task-4").unwrap();
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn where_no_harness_is_benchmarked_in_full_the_primogenitor_is_the_best() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    attempt(&mut genealogy, 0, 1, 0);
    attempt(&mut genealogy, 1, 3, 3);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_failed_harness_is_never_the_best() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Failed);
    benchmarked_in_full(&mut genealogy, 0, 0);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_harness_that_is_being_created_is_never_the_best() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    genealogy.start_procreation(0).unwrap();
    benchmarked_in_full(&mut genealogy, 0, 0);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn the_best_harness_is_judged_by_its_own_results_and_not_its_clade() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    child_of(&mut genealogy, 0, CreationOutcome::Created);
    benchmarked_in_full(&mut genealogy, 0, 1);
    benchmarked_in_full(&mut genealogy, 1, 3);
    benchmarked_in_full(&mut genealogy, 2, 2);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_primogenitor_that_awaits_its_build_is_the_best_harness() {
    assert_eq!(best(&Genealogy::unbuilt()), 0);
}
