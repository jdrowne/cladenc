use crate::genealogy::{Cause, CreationOutcome, Genealogy, HarnessId, TrialResult};

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
    let child = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    genealogy
}

fn after(harness: HarnessId, task: &str, result: TrialResult) -> Genealogy {
    let mut genealogy = with_a_child();
    genealogy.start_trial(harness, task).unwrap();
    genealogy.end_trial(harness, task, result).unwrap();
    genealogy
}

fn counts_of_the_child(genealogy: &Genealogy) -> [u64; 9] {
    let child = &genealogy.harnesses()[1];
    [
        child.selection_passed(),
        child.selection_failed(),
        child.selection_inconclusive(),
        child.corroboration_passed(),
        child.corroboration_failed(),
        child.corroboration_inconclusive(),
        child.non_holdout_passed(),
        child.non_holdout_failed(),
        child.non_holdout_inconclusive(),
    ]
}

fn clade_passed_of_the_line(genealogy: &Genealogy) -> [u64; 2] {
    [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_passed())
}

fn clade_failed_of_the_line(genealogy: &Genealogy) -> [u64; 2] {
    [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_failed())
}

fn clade_inconclusive_of_the_line(genealogy: &Genealogy) -> [u64; 2] {
    [0, 1].map(|harness: usize| genealogy.harnesses()[harness].clade_inconclusive())
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
    assert_eq!(
        counts_of_the_child(&with_a_child()),
        [0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_trial_that_passed_on_a_selection_task_adds_one_to_the_selection_passed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SELECTION, TrialResult::Pass)),
        [1, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_failed_trial_at_a_selection_task_adds_one_to_the_selection_failed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SELECTION, TrialResult::Fail)),
        [0, 1, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_selection_task_adds_one_to_the_selection_inconclusive_alone()
 {
    assert_eq!(
        counts_of_the_child(&after(
            1,
            SELECTION,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [0, 0, 1, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_trial_whose_verifier_timed_out_at_a_selection_task_adds_one_to_the_selection_inconclusive_alone()
 {
    assert_eq!(
        counts_of_the_child(&after(
            1,
            SELECTION,
            TrialResult::Inconclusive(Cause::TimedOut)
        )),
        [0, 0, 1, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_trial_that_passed_on_a_corroboration_task_adds_one_to_the_corroboration_passed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, CORROBORATION, TrialResult::Pass)),
        [0, 0, 0, 1, 0, 0, 0, 0, 0]
    );
}

#[test]
fn a_failed_trial_at_a_corroboration_task_adds_one_to_the_corroboration_failed() {
    assert_eq!(
        counts_of_the_child(&after(1, CORROBORATION, TrialResult::Fail)),
        [0, 0, 0, 0, 1, 0, 0, 0, 0]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_corroboration_task_adds_one_to_the_corroboration_inconclusive()
 {
    assert_eq!(
        counts_of_the_child(&after(
            1,
            CORROBORATION,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [0, 0, 0, 0, 0, 1, 0, 0, 0]
    );
}

#[test]
fn a_trial_whose_verifier_timed_out_at_a_corroboration_task_adds_one_to_the_corroboration_inconclusive()
 {
    assert_eq!(
        counts_of_the_child(&after(
            1,
            CORROBORATION,
            TrialResult::Inconclusive(Cause::TimedOut)
        )),
        [0, 0, 0, 0, 0, 1, 0, 0, 0]
    );
}

#[test]
fn a_trial_that_passed_on_a_task_not_held_out_adds_one_to_the_non_holdout_passed_alone() {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, TrialResult::Pass)),
        [0, 0, 0, 0, 0, 0, 1, 0, 0]
    );
}

#[test]
fn a_failed_trial_at_a_task_not_held_out_adds_one_to_the_non_holdout_failed() {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, TrialResult::Fail)),
        [0, 0, 0, 0, 0, 0, 0, 1, 0]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_task_not_held_out_adds_one_to_the_non_holdout_inconclusive()
 {
    assert_eq!(
        counts_of_the_child(&after(
            1,
            SHOWN,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [0, 0, 0, 0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn a_trial_whose_verifier_timed_out_at_a_task_not_held_out_adds_one_to_the_non_holdout_inconclusive()
 {
    assert_eq!(
        counts_of_the_child(&after(1, SHOWN, TrialResult::Inconclusive(Cause::TimedOut))),
        [0, 0, 0, 0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn a_trial_that_passed_on_a_selection_task_reaches_the_clade_passed_of_its_line() {
    assert_eq!(
        clade_passed_of_the_line(&after(1, SELECTION, TrialResult::Pass)),
        [1, 1]
    );
}

#[test]
fn a_failed_trial_at_a_selection_task_reaches_the_clade_failed_of_its_line() {
    assert_eq!(
        clade_failed_of_the_line(&after(1, SELECTION, TrialResult::Fail)),
        [1, 1]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_selection_task_reaches_the_clade_inconclusive_of_its_line()
 {
    assert_eq!(
        clade_inconclusive_of_the_line(&after(
            1,
            SELECTION,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [1, 1]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_corroboration_task_reaches_no_clade_inconclusive() {
    assert_eq!(
        clade_inconclusive_of_the_line(&after(
            1,
            CORROBORATION,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [0, 0]
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_at_a_task_that_is_not_held_out_reaches_no_clade_inconclusive()
 {
    assert_eq!(
        clade_inconclusive_of_the_line(&after(
            1,
            SHOWN,
            TrialResult::Inconclusive(Cause::NoVerdict)
        )),
        [0, 0]
    );
}

#[test]
fn a_trial_that_passed_on_a_corroboration_task_reaches_no_clade_passed() {
    assert_eq!(
        clade_passed_of_the_line(&after(1, CORROBORATION, TrialResult::Pass)),
        [0, 0]
    );
}

#[test]
fn a_failed_trial_at_a_corroboration_task_reaches_no_clade_failed() {
    assert_eq!(
        clade_failed_of_the_line(&after(1, CORROBORATION, TrialResult::Fail)),
        [0, 0]
    );
}

#[test]
fn a_clade_that_finished_only_corroboration_tasks_has_no_clade_completion_rate() {
    let genealogy = after(1, CORROBORATION, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), None);
}

#[test]
fn a_trial_that_passed_on_a_task_that_is_not_held_out_reaches_no_clade_passed() {
    assert_eq!(
        clade_passed_of_the_line(&after(1, SHOWN, TrialResult::Pass)),
        [0, 0]
    );
}

#[test]
fn a_failed_trial_at_a_task_that_is_not_held_out_reaches_no_clade_failed() {
    assert_eq!(
        clade_failed_of_the_line(&after(1, SHOWN, TrialResult::Fail)),
        [0, 0]
    );
}

#[test]
fn a_clade_that_finished_only_tasks_that_are_not_held_out_has_no_clade_completion_rate() {
    let genealogy = after(1, SHOWN, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), None);
}

#[test]
fn a_genealogy_told_of_no_task_counts_every_task_as_one_that_is_not_held_out() {
    let mut genealogy = Genealogy::new();
    genealogy.start_trial(0, SELECTION).unwrap();
    genealogy
        .end_trial(0, SELECTION, TrialResult::Pass)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].non_holdout_passed(), 1);
}

#[test]
fn a_trial_on_a_selection_task_is_started_and_counted_as_any_trial_is() {
    assert_eq!(after(1, SELECTION, TrialResult::Pass).started(), 1);
}

#[test]
fn a_trial_on_a_corroboration_task_is_started_and_counted_as_any_trial_is() {
    assert_eq!(after(1, CORROBORATION, TrialResult::Pass).started(), 1);
}

#[test]
fn a_harness_has_completed_a_corroboration_task_it_completed() {
    let genealogy = after(1, CORROBORATION, TrialResult::Pass);
    assert!(genealogy.completed_by_any(CORROBORATION));
}
