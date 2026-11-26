use crate::genealogy::{CreationOutcome, Genealogy, GenealogyError, TrialResult};

fn with_a_child_being_created() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_diversification(0).unwrap();
    genealogy
}

fn with_a_failed_child() -> Genealogy {
    let mut genealogy = with_a_child_being_created();
    genealogy
        .end_diversification(1, CreationOutcome::Failed)
        .unwrap();
    genealogy
}

fn with_an_unchanged_child() -> Genealogy {
    let mut genealogy = with_a_child_being_created();
    genealogy
        .end_diversification(1, CreationOutcome::Unchanged)
        .unwrap();
    genealogy
}

fn with_a_trial_running() -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_trial(0, "a").unwrap();
    genealogy
}

fn with_a_trial_ended() -> Genealogy {
    let mut genealogy = with_a_trial_running();
    genealogy.end_trial(0, "a", TrialResult::Completed).unwrap();
    genealogy
}

fn not_running(harness: u64, task: &str) -> GenealogyError {
    GenealogyError::TrialNotRunning {
        harness,
        task: task.to_owned(),
    }
}

fn already_given(harness: u64, task: &str) -> GenealogyError {
    GenealogyError::TaskAlreadyGiven {
        harness,
        task: task.to_owned(),
    }
}

#[test]
fn a_trial_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().start_trial(7, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 7 })
    );
}

#[test]
fn a_refused_trial_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.start_trial(7, "a");
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn a_trial_of_a_harness_that_is_being_created_is_refused() {
    assert_eq!(
        with_a_child_being_created().start_trial(1, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 1 })
    );
}

#[test]
fn a_refused_trial_of_a_harness_being_created_changes_nothing() {
    let mut genealogy = with_a_child_being_created();
    let _ = genealogy.start_trial(1, "a");
    assert_eq!(genealogy, with_a_child_being_created());
}

#[test]
fn a_trial_of_a_failed_harness_is_refused() {
    assert_eq!(
        with_a_failed_child().start_trial(1, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 1 })
    );
}

#[test]
fn a_refused_trial_of_a_failed_harness_changes_nothing() {
    let mut genealogy = with_a_failed_child();
    let _ = genealogy.start_trial(1, "a");
    assert_eq!(genealogy, with_a_failed_child());
}

#[test]
fn a_trial_of_an_unchanged_harness_is_refused() {
    assert_eq!(
        with_an_unchanged_child().start_trial(1, "a"),
        Err(GenealogyError::HarnessNotCreated { harness: 1 })
    );
}

#[test]
fn a_second_trial_on_a_task_whose_trial_runs_is_refused() {
    assert_eq!(
        with_a_trial_running().start_trial(0, "a"),
        Err(already_given(0, "a"))
    );
}

#[test]
fn a_refused_second_trial_on_a_running_task_changes_nothing() {
    let mut genealogy = with_a_trial_running();
    let _ = genealogy.start_trial(0, "a");
    assert_eq!(genealogy, with_a_trial_running());
}

#[test]
fn a_second_trial_on_a_task_whose_trial_ended_is_refused() {
    assert_eq!(
        with_a_trial_ended().start_trial(0, "a"),
        Err(already_given(0, "a"))
    );
}

#[test]
fn a_refused_second_trial_on_an_ended_task_changes_nothing() {
    let mut genealogy = with_a_trial_ended();
    let _ = genealogy.start_trial(0, "a");
    assert_eq!(genealogy, with_a_trial_ended());
}

#[test]
fn the_end_of_a_trial_of_a_harness_that_is_not_in_the_genealogy_is_refused() {
    assert_eq!(
        Genealogy::new().end_trial(7, "a", TrialResult::Completed),
        Err(not_running(7, "a"))
    );
}

#[test]
fn a_refused_end_of_a_trial_of_a_missing_harness_changes_nothing() {
    let mut genealogy = Genealogy::new();
    let _ = genealogy.end_trial(7, "a", TrialResult::Completed);
    assert_eq!(genealogy, Genealogy::new());
}

#[test]
fn the_end_of_a_trial_on_a_task_never_given_is_refused() {
    assert_eq!(
        with_a_trial_running().end_trial(0, "b", TrialResult::Completed),
        Err(not_running(0, "b"))
    );
}

#[test]
fn a_refused_end_of_a_trial_on_a_task_never_given_changes_nothing() {
    let mut genealogy = with_a_trial_running();
    let _ = genealogy.end_trial(0, "b", TrialResult::Completed);
    assert_eq!(genealogy, with_a_trial_running());
}

#[test]
fn a_second_end_of_a_trial_is_refused() {
    assert_eq!(
        with_a_trial_ended().end_trial(0, "a", TrialResult::NotCompleted),
        Err(not_running(0, "a"))
    );
}

#[test]
fn a_refused_second_end_of_a_trial_changes_nothing() {
    let mut genealogy = with_a_trial_ended();
    let _ = genealogy.end_trial(0, "a", TrialResult::NotCompleted);
    assert_eq!(genealogy, with_a_trial_ended());
}
