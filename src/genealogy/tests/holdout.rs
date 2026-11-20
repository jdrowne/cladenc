use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy, HarnessId};

const SELECTION: &str = "selection";
const CORROBORATION: &str = "corroboration";
const SHOWN: &str = "shown";

fn told_of_its_tasks() -> Genealogy {
    Genealogy::new()
        .selecting_by(&[SELECTION.to_owned()])
        .corroborating_with(&[CORROBORATION.to_owned()])
}

fn with_a_child() -> Genealogy {
    let mut genealogy = told_of_its_tasks();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    genealogy
}

fn after(harness: HarnessId, task: &str, result: AttemptResult) -> Genealogy {
    let mut genealogy = with_a_child();
    genealogy.start_attempt(harness, task).unwrap();
    genealogy.end_attempt(harness, task, result).unwrap();
    genealogy
}

fn counts_of_the_child(genealogy: &Genealogy) -> [u64; 6] {
    let child = &genealogy.harnesses()[1];
    [
        child.selection_completed(),
        child.selection_not_completed(),
        child.corroboration_completed(),
        child.corroboration_not_completed(),
        child.other_completed(),
        child.other_not_completed(),
    ]
}

fn clade_completed_of_the_line(genealogy: &Genealogy) -> [u64; 2] {
    [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_completed())
}

fn clade_not_completed_of_the_line(genealogy: &Genealogy) -> [u64; 2] {
    [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_not_completed())
}

#[test]
fn a_genealogy_told_of_its_tasks_still_holds_the_primogenitor_alone() {
    assert_eq!(told_of_its_tasks().harness_count(), 1);
}

#[test]
fn a_genealogy_that_selects_by_a_task_is_not_one_that_selects_by_none() {
    assert_ne!(
        Genealogy::new().selecting_by(&[SELECTION.to_owned()]),
        Genealogy::new()
    );
}

#[test]
fn a_genealogy_that_corroborates_with_a_task_is_not_one_that_corroborates_with_none() {
    assert_ne!(
        Genealogy::new().corroborating_with(&[CORROBORATION.to_owned()]),
        Genealogy::new()
    );
}

#[test]
fn a_genealogy_that_corroborates_with_a_task_is_not_one_that_selects_by_the_task() {
    assert_ne!(
        Genealogy::new().corroborating_with(&[SELECTION.to_owned()]),
        Genealogy::new().selecting_by(&[SELECTION.to_owned()])
    );
}

#[test]
fn a_new_harness_has_no_count_of_any_task() {
    assert_eq!(counts_of_the_child(&with_a_child()), [0, 0, 0, 0, 0, 0]);
}

#[test]
fn a_completed_attempt_at_a_selection_task_adds_one_to_the_selection_completed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SELECTION, AttemptResult::Completed)),
        [1, 0, 0, 0, 0, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_selection_task_adds_one_to_the_selection_not_completed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SELECTION, AttemptResult::NotCompleted)),
        [0, 1, 0, 0, 0, 0]
    );
}

#[test]
fn a_result_of_timed_out_at_a_selection_task_counts_as_selection_not_completed() {
    assert_eq!(
        counts_of_the_child(&after(1, SELECTION, AttemptResult::TimedOut)),
        [0, 1, 0, 0, 0, 0]
    );
}

#[test]
fn a_completed_attempt_at_a_corroboration_task_adds_one_to_the_corroboration_completed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, CORROBORATION, AttemptResult::Completed)),
        [0, 0, 1, 0, 0, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_corroboration_task_adds_one_to_the_corroboration_not_completed() {
    assert_eq!(
        counts_of_the_child(&after(1, CORROBORATION, AttemptResult::NotCompleted)),
        [0, 0, 0, 1, 0, 0]
    );
}

#[test]
fn a_result_of_timed_out_at_a_corroboration_task_counts_as_corroboration_not_completed() {
    assert_eq!(
        counts_of_the_child(&after(1, CORROBORATION, AttemptResult::TimedOut)),
        [0, 0, 0, 1, 0, 0]
    );
}

#[test]
fn a_completed_attempt_at_a_task_that_is_not_held_out_adds_one_to_the_other_completed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, AttemptResult::Completed)),
        [0, 0, 0, 0, 1, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_task_that_is_not_held_out_adds_one_to_the_other_not_completed() {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, AttemptResult::NotCompleted)),
        [0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn a_result_of_timed_out_at_a_task_that_is_not_held_out_counts_as_other_not_completed() {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, AttemptResult::TimedOut)),
        [0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn a_completed_attempt_at_a_selection_task_reaches_the_clade_completed_of_its_line() {
    assert_eq!(
        clade_completed_of_the_line(&after(1, SELECTION, AttemptResult::Completed)),
        [1, 1]
    );
}

#[test]
fn an_attempt_not_completed_at_a_selection_task_reaches_the_clade_not_completed_of_its_line() {
    assert_eq!(
        clade_not_completed_of_the_line(&after(1, SELECTION, AttemptResult::NotCompleted)),
        [1, 1]
    );
}

#[test]
fn a_completed_attempt_at_a_corroboration_task_reaches_no_clade_completed() {
    assert_eq!(
        clade_completed_of_the_line(&after(1, CORROBORATION, AttemptResult::Completed)),
        [0, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_corroboration_task_reaches_no_clade_not_completed() {
    assert_eq!(
        clade_not_completed_of_the_line(&after(1, CORROBORATION, AttemptResult::NotCompleted)),
        [0, 0]
    );
}

#[test]
fn a_clade_that_finished_only_corroboration_tasks_has_no_clade_score() {
    let genealogy = after(1, CORROBORATION, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_score(), None);
}

#[test]
fn a_completed_attempt_at_a_task_that_is_not_held_out_reaches_no_clade_completed() {
    assert_eq!(
        clade_completed_of_the_line(&after(1, SHOWN, AttemptResult::Completed)),
        [0, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_task_that_is_not_held_out_reaches_no_clade_not_completed() {
    assert_eq!(
        clade_not_completed_of_the_line(&after(1, SHOWN, AttemptResult::NotCompleted)),
        [0, 0]
    );
}

#[test]
fn a_clade_that_finished_only_tasks_that_are_not_held_out_has_no_clade_score() {
    let genealogy = after(1, SHOWN, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_score(), None);
}

#[test]
fn a_genealogy_told_of_no_task_counts_every_task_as_one_that_is_not_held_out() {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, SELECTION).unwrap();
    genealogy
        .end_attempt(0, SELECTION, AttemptResult::Completed)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].other_completed(), 1);
}

#[test]
fn an_attempt_at_a_selection_task_is_started_and_counted_as_any_attempt_is() {
    assert_eq!(after(1, SELECTION, AttemptResult::Completed).started(), 1);
}

#[test]
fn an_attempt_at_a_corroboration_task_is_started_and_counted_as_any_attempt_is() {
    assert_eq!(
        after(1, CORROBORATION, AttemptResult::Completed).started(),
        1
    );
}

#[test]
fn a_harness_has_completed_a_corroboration_task_it_completed() {
    let genealogy = after(1, CORROBORATION, AttemptResult::Completed);
    assert!(genealogy.completed_by_any(CORROBORATION));
}
