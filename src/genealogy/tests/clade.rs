use crate::genealogy::{Cause, CreationOutcome, Genealogy, HarnessId, TrialResult};

fn family() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&["a".to_owned()]);
    for progenitor in [0, 1, 0] {
        let child = genealogy.start_diversification(progenitor).unwrap();
        genealogy
            .end_diversification(child, CreationOutcome::Created)
            .unwrap();
    }
    genealogy
}

fn finish(genealogy: &mut Genealogy, harness: HarnessId, result: TrialResult) {
    genealogy.start_trial(harness, "a").unwrap();
    genealogy.end_trial(harness, "a", result).unwrap();
}

fn family_after(harness: HarnessId, result: TrialResult) -> Genealogy {
    let mut genealogy = family();
    finish(&mut genealogy, harness, result);
    genealogy
}

fn family_with_every_harness_finishing(result: TrialResult) -> Genealogy {
    let mut genealogy = family();
    for harness in 0..4 {
        finish(&mut genealogy, harness, result);
    }
    genealogy
}

#[test]
fn a_trial_that_passed_reaches_the_clade_passed_of_its_own_harness() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[2].clade_passed(), 1);
}

#[test]
fn a_trial_that_passed_reaches_the_clade_passed_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[1].clade_passed(), 1);
}

#[test]
fn a_trial_that_passed_reaches_the_clade_passed_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_passed(), 1);
}

#[test]
fn a_failed_trial_reaches_the_clade_failed_of_its_own_harness() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[2].clade_failed(), 1);
}

#[test]
fn a_failed_trial_reaches_the_clade_failed_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 1);
}

#[test]
fn a_failed_trial_reaches_the_clade_failed_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].clade_failed(), 1);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_reaches_the_clade_inconclusive_of_its_own_harness() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[2].clade_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_reaches_the_clade_inconclusive_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[1].clade_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_reaches_the_clade_inconclusive_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].clade_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_adds_nothing_to_the_clade_failed() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].clade_failed(), 0);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_adds_nothing_to_the_clade_passed() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[0].clade_passed(), 0);
}

#[test]
fn a_trial_whose_verifier_timed_out_reaches_the_clade_inconclusive_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::TimedOut));
    assert_eq!(genealogy.harnesses()[0].clade_inconclusive(), 1);
}

#[test]
fn a_trial_whose_verifier_timed_out_adds_nothing_to_the_clade_failed() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::TimedOut));
    assert_eq!(genealogy.harnesses()[0].clade_failed(), 0);
}

#[test]
fn a_trial_that_passed_adds_nothing_to_the_clade_inconclusive() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_inconclusive(), 0);
}

#[test]
fn a_failed_trial_adds_nothing_to_the_clade_inconclusive() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].clade_inconclusive(), 0);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_stays_out_of_the_clade_inconclusive_of_a_sibling_of_its_progenitor()
 {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[3].clade_inconclusive(), 0);
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_stays_out_of_the_clade_inconclusive_of_a_child() {
    let genealogy = family_after(1, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[2].clade_inconclusive(), 0);
}

#[test]
fn a_trial_that_passed_adds_nothing_to_the_clade_failed() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_failed(), 0);
}

#[test]
fn a_failed_trial_adds_nothing_to_the_clade_passed() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].clade_passed(), 0);
}

#[test]
fn a_trial_that_passed_stays_out_of_the_clade_passed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[3].clade_passed(), 0);
}

#[test]
fn a_trial_that_passed_stays_out_of_the_clade_passed_of_a_child() {
    let genealogy = family_after(1, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[2].clade_passed(), 0);
}

#[test]
fn a_failed_trial_stays_out_of_the_clade_failed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[3].clade_failed(), 0);
}

#[test]
fn a_failed_trial_stays_out_of_the_clade_failed_of_a_child() {
    let genealogy = family_after(1, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[2].clade_failed(), 0);
}

fn family_after_a_diversification_that_created_no_child(outcome: CreationOutcome) -> Genealogy {
    let mut genealogy = family_after(1, TrialResult::Fail);
    let child = genealogy.start_diversification(1).unwrap();
    genealogy.end_diversification(child, outcome).unwrap();
    genealogy
}

#[test]
fn a_diversification_that_failed_adds_nothing_to_the_clade_failed_of_its_progenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 1);
}

#[test]
fn a_diversification_whose_child_did_not_build_adds_nothing_to_the_clade_failed_of_its_progenitor()
{
    let genealogy =
        family_after_a_diversification_that_created_no_child(CreationOutcome::BuildFailed);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 1);
}

#[test]
fn a_diversification_that_timed_out_adds_nothing_to_the_clade_failed_of_its_progenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 1);
}

#[test]
fn a_diversification_that_left_the_progenitor_s_clone_unchanged_adds_nothing_to_the_clade_of_its_progenitor()
 {
    let genealogy =
        family_after_a_diversification_that_created_no_child(CreationOutcome::Unchanged);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 1);
}

#[test]
fn a_diversification_that_failed_adds_nothing_to_the_clade_failed_of_the_primogenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[0].clade_failed(), 1);
}

#[test]
fn the_clade_completed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[1].clade_passed(), 2);
}

#[test]
fn the_clade_failed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[1].clade_failed(), 2);
}

#[test]
fn the_clade_inconclusive_are_summed_over_the_clade() {
    let genealogy =
        family_with_every_harness_finishing(TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[1].clade_inconclusive(), 2);
}

#[test]
fn the_clade_of_the_primogenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_passed(), 4);
}

#[test]
fn the_clade_completion_rate_is_the_share_completed() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, TrialResult::Pass);
    finish(&mut genealogy, 1, TrialResult::Pass);
    finish(&mut genealogy, 2, TrialResult::Pass);
    finish(&mut genealogy, 3, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), Some(0.75));
}

#[test]
fn an_inconclusive_trial_is_one_of_the_finished_trials_the_clade_completion_rate_is_over() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, TrialResult::Pass);
    finish(&mut genealogy, 1, TrialResult::Pass);
    finish(&mut genealogy, 2, TrialResult::Pass);
    finish(
        &mut genealogy,
        3,
        TrialResult::Inconclusive(Cause::NoVerdict),
    );
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), Some(0.75));
}

#[test]
fn a_trial_whose_verifier_timed_out_is_one_of_the_finished_trials_the_clade_completion_rate_is_over()
 {
    let mut genealogy = family();
    finish(&mut genealogy, 0, TrialResult::Pass);
    finish(
        &mut genealogy,
        1,
        TrialResult::Inconclusive(Cause::TimedOut),
    );
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), Some(0.5));
}

#[test]
fn the_clade_completion_rate_of_a_clade_whose_only_trial_is_inconclusive_is_zero() {
    let genealogy = family_after(2, TrialResult::Inconclusive(Cause::NoVerdict));
    assert_eq!(genealogy.harnesses()[1].clade_completion_rate(), Some(0.0));
}

#[test]
fn the_clade_completion_rate_of_a_clade_with_nothing_completed_is_zero() {
    let genealogy = family_after(2, TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[1].clade_completion_rate(), Some(0.0));
}

#[test]
fn the_clade_completion_rate_has_no_value_in_a_new_genealogy() {
    assert_eq!(
        Genealogy::new().harnesses()[0].clade_completion_rate(),
        None
    );
}

#[test]
fn the_clade_completion_rate_has_no_value_while_the_only_trial_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_trial(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), None);
}

#[test]
fn the_clade_completion_rate_has_no_value_for_a_clade_the_results_fall_outside() {
    let genealogy = family_after(3, TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[1].clade_completion_rate(), None);
}
