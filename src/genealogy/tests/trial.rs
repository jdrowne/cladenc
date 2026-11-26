use crate::genealogy::{Cause, CreationOutcome, Genealogy, TrialResult};

fn selecting_by_both_tasks() -> Genealogy {
    Genealogy::new().selecting_by(&["a".to_owned(), "b".to_owned()])
}

fn trialled(result: TrialResult) -> Genealogy {
    let mut genealogy = selecting_by_both_tasks();
    genealogy.start_trial(0, "a").unwrap();
    genealogy.end_trial(0, "a", result).unwrap();
    genealogy
}

#[test]
fn started_counts_a_trial_that_has_not_ended() {
    let mut genealogy = Genealogy::new();
    genealogy.start_trial(0, "a").unwrap();
    assert_eq!(genealogy.started(), 1);
}

#[test]
fn started_keeps_counting_a_trial_that_has_ended() {
    assert_eq!(trialled(TrialResult::Pass).started(), 1);
}

#[test]
fn started_counts_the_trials_of_every_harness() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    genealogy.start_trial(0, "a").unwrap();
    genealogy.start_trial(0, "b").unwrap();
    genealogy.start_trial(child, "a").unwrap();
    assert_eq!(genealogy.started(), 3);
}

#[test]
fn a_harness_whose_trial_has_not_ended_counts_as_neither_passed_nor_failed() {
    let mut genealogy = Genealogy::new();
    genealogy.start_trial(0, "a").unwrap();
    let primogenitor = &genealogy.harnesses()[0];
    assert_eq!(
        (
            primogenitor.selection_passed(),
            primogenitor.selection_failed()
        ),
        (0, 0)
    );
}

#[test]
fn a_trial_that_passed_adds_one_to_the_passed_of_its_harness() {
    let genealogy = trialled(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].selection_passed(), 1);
}

#[test]
fn a_trial_that_passed_adds_nothing_to_the_failed_of_its_harness() {
    let genealogy = trialled(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].selection_failed(), 0);
}

#[test]
fn a_failed_trial_adds_one_to_the_failed_of_its_harness() {
    let genealogy = trialled(TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].selection_failed(), 1);
}

#[test]
fn a_failed_trial_adds_nothing_to_the_passed_of_its_harness() {
    let genealogy = trialled(TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].selection_passed(), 0);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_adds_one_to_the_inconclusive_of_its_harness() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].selection_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_adds_nothing_to_the_failed() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].selection_failed(), 0);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_adds_nothing_to_the_passed() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].selection_passed(), 0);
}

#[test]
fn a_trial_whose_verifier_timed_out_adds_one_to_the_inconclusive_of_its_harness() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::TimedOut));
    assert_eq!(genealogy.harnesses()[0].selection_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_timed_out_adds_nothing_to_the_failed() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::TimedOut));
    assert_eq!(genealogy.harnesses()[0].selection_failed(), 0);
}

#[test]
fn a_trial_whose_verifier_timed_out_adds_nothing_to_the_passed() {
    let genealogy = trialled(TrialResult::Inconclusive(Cause::TimedOut));
    assert_eq!(genealogy.harnesses()[0].selection_passed(), 0);
}

#[test]
fn a_trial_that_passed_adds_nothing_to_the_inconclusive_of_its_harness() {
    let genealogy = trialled(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].selection_inconclusive(), 0);
}

#[test]
fn a_failed_trial_adds_nothing_to_the_inconclusive_of_its_harness() {
    let genealogy = trialled(TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].selection_inconclusive(), 0);
}

#[test]
fn the_results_of_a_harness_add_up_over_its_tasks() {
    let mut genealogy = trialled(TrialResult::Pass);
    genealogy.start_trial(0, "b").unwrap();
    genealogy.end_trial(0, "b", TrialResult::Pass).unwrap();
    assert_eq!(genealogy.harnesses()[0].selection_passed(), 2);
}
