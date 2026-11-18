use crate::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

const HELD_OUT: &str = "held-out";
const SHOWN: &str = "shown";

fn with_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new().holding_out(&[HELD_OUT.to_owned()]);
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy
}

fn after(harness: HarnessId, task: &str, result: AttemptResult) -> Genealogy {
    let mut genealogy = with_a_child();
    genealogy.start_attempt(harness, task).unwrap();
    genealogy.end_attempt(harness, task, result).unwrap();
    genealogy
}

#[test]
fn a_genealogy_told_of_its_holdout_tasks_still_holds_the_primogenitor_alone() {
    assert_eq!(
        Genealogy::new()
            .holding_out(&[HELD_OUT.to_owned()])
            .harness_count(),
        1
    );
}

#[test]
fn a_genealogy_that_holds_a_task_out_is_not_one_that_holds_none_out() {
    assert_ne!(
        Genealogy::new().holding_out(&[HELD_OUT.to_owned()]),
        Genealogy::new()
    );
}

#[test]
fn a_completed_attempt_at_a_holdout_task_adds_one_to_the_holdout_completed_of_its_harness() {
    let genealogy = after(1, HELD_OUT, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].holdout_completed(), 1);
}

#[test]
fn a_completed_attempt_at_a_holdout_task_adds_nothing_to_the_other_completed_of_its_harness() {
    let genealogy = after(1, HELD_OUT, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].other_completed(), 0);
}

#[test]
fn a_completed_attempt_at_a_task_that_is_not_held_out_adds_one_to_the_other_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].other_completed(), 1);
}

#[test]
fn a_completed_attempt_at_a_task_that_is_not_held_out_adds_nothing_to_the_holdout_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].holdout_completed(), 0);
}

#[test]
fn an_attempt_not_completed_at_a_task_that_is_not_held_out_adds_one_to_the_other_not_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].other_not_completed(), 1);
}

#[test]
fn an_attempt_not_completed_at_a_task_that_is_not_held_out_leaves_the_holdout_not_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].holdout_not_completed(), 0);
}

#[test]
fn a_result_of_timed_out_at_a_task_that_is_not_held_out_counts_as_other_not_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::TimedOut);
    assert_eq!(genealogy.harnesses()[1].other_not_completed(), 1);
}

#[test]
fn a_new_harness_has_completed_no_task_that_is_not_held_out() {
    assert_eq!(with_a_child().harnesses()[1].other_completed(), 0);
}

#[test]
fn a_new_harness_has_no_task_that_is_not_held_out_not_completed() {
    assert_eq!(with_a_child().harnesses()[1].other_not_completed(), 0);
}

#[test]
fn a_completed_attempt_at_a_task_that_is_not_held_out_reaches_no_clade_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::Completed);
    assert_eq!(
        [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_completed()),
        [0, 0]
    );
}

#[test]
fn an_attempt_not_completed_at_a_task_that_is_not_held_out_reaches_no_clade_not_completed() {
    let genealogy = after(1, SHOWN, AttemptResult::NotCompleted);
    assert_eq!(
        [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_not_completed()),
        [0, 0]
    );
}

#[test]
fn a_clade_that_finished_only_tasks_that_are_not_held_out_has_no_clade_score() {
    let genealogy = after(1, SHOWN, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_score(), None);
}

#[test]
fn a_completed_attempt_at_a_holdout_task_reaches_the_clade_completed_of_its_line() {
    let genealogy = after(1, HELD_OUT, AttemptResult::Completed);
    assert_eq!(
        [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_completed()),
        [1, 1]
    );
}

#[test]
fn a_genealogy_that_holds_no_task_out_counts_every_task_as_one_that_is_not_held_out() {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, HELD_OUT).unwrap();
    genealogy
        .end_attempt(0, HELD_OUT, AttemptResult::Completed)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].other_completed(), 1);
}

#[test]
fn an_attempt_at_a_holdout_task_is_started_and_counted_as_any_attempt_is() {
    assert_eq!(after(1, HELD_OUT, AttemptResult::Completed).started(), 1);
}
