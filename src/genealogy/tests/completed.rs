use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

fn with_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    genealogy
}

fn after(harness: HarnessId, result: TrialResult) -> Genealogy {
    let mut genealogy = with_a_child();
    genealogy.start_trial(harness, "a").unwrap();
    genealogy.end_trial(harness, "a", result).unwrap();
    genealogy
}

#[test]
fn in_a_new_genealogy_no_harness_has_completed_a_task() {
    assert!(!Genealogy::new().completed_by_any("a"));
}

#[test]
fn a_task_the_primogenitor_completed_is_completed_by_one_harness_at_least() {
    assert!(after(0, TrialResult::Completed).completed_by_any("a"));
}

#[test]
fn a_task_a_child_completed_is_completed_by_one_harness_at_least() {
    assert!(after(1, TrialResult::Completed).completed_by_any("a"));
}

#[test]
fn a_task_that_was_trialled_and_not_completed_is_completed_by_no_harness() {
    assert!(!after(0, TrialResult::NotCompleted).completed_by_any("a"));
}

#[test]
fn a_task_whose_verifier_left_no_verdict_is_completed_by_no_harness() {
    assert!(!after(0, TrialResult::NoVerdict).completed_by_any("a"));
}

#[test]
fn a_task_whose_trial_ran_past_its_time_limit_is_completed_by_no_harness() {
    assert!(!after(0, TrialResult::TimedOut).completed_by_any("a"));
}

#[test]
fn a_task_whose_trial_has_not_ended_is_completed_by_no_harness() {
    let mut genealogy = with_a_child();
    genealogy.start_trial(0, "a").unwrap();
    assert!(!genealogy.completed_by_any("a"));
}

#[test]
fn a_task_no_harness_trialled_is_completed_by_no_harness() {
    assert!(!after(0, TrialResult::Completed).completed_by_any("b"));
}

#[test]
fn the_harness_that_completed_a_task_has_completed_it() {
    assert!(after(1, TrialResult::Completed).harnesses()[1].has_completed("a"));
}

#[test]
fn a_harness_has_not_completed_a_task_that_another_harness_completed() {
    assert!(!after(1, TrialResult::Completed).harnesses()[0].has_completed("a"));
}
