use cladenc::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

fn with_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy
}

fn after(harness: HarnessId, result: AttemptResult) -> Genealogy {
    let mut genealogy = with_a_child();
    genealogy.start_attempt(harness, "a").unwrap();
    genealogy.end_attempt(harness, "a", result).unwrap();
    genealogy
}

#[test]
fn in_a_new_genealogy_no_harness_has_completed_a_task() {
    assert!(!Genealogy::new().completed_by_any("a"));
}

#[test]
fn a_task_the_primogenitor_completed_is_completed_by_one_harness_at_least() {
    assert!(after(0, AttemptResult::Completed).completed_by_any("a"));
}

#[test]
fn a_task_a_child_completed_is_completed_by_one_harness_at_least() {
    assert!(after(1, AttemptResult::Completed).completed_by_any("a"));
}

#[test]
fn a_task_that_was_attempted_and_not_completed_is_completed_by_no_harness() {
    assert!(!after(0, AttemptResult::NotCompleted).completed_by_any("a"));
}

#[test]
fn a_task_whose_attempt_ran_past_its_time_limit_is_completed_by_no_harness() {
    assert!(!after(0, AttemptResult::TimedOut).completed_by_any("a"));
}

#[test]
fn a_task_whose_attempt_has_not_ended_is_completed_by_no_harness() {
    let mut genealogy = with_a_child();
    genealogy.start_attempt(0, "a").unwrap();
    assert!(!genealogy.completed_by_any("a"));
}

#[test]
fn a_task_no_harness_attempted_is_completed_by_no_harness() {
    assert!(!after(0, AttemptResult::Completed).completed_by_any("b"));
}

#[test]
fn the_harness_that_completed_a_task_has_completed_it() {
    assert!(after(1, AttemptResult::Completed).harnesses()[1].has_completed("a"));
}

#[test]
fn a_harness_has_not_completed_a_task_that_another_harness_completed() {
    assert!(!after(1, AttemptResult::Completed).harnesses()[0].has_completed("a"));
}
