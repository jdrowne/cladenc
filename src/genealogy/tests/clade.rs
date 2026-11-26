use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

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
fn a_completed_trial_reaches_the_clade_completed_of_its_own_harness() {
    let genealogy = family_after(2, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[2].clade_completed(), 1);
}

#[test]
fn a_completed_trial_reaches_the_clade_completed_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_completed(), 1);
}

#[test]
fn a_completed_trial_reaches_the_clade_completed_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 1);
}

#[test]
fn a_trial_not_completed_reaches_the_clade_not_completed_of_its_own_harness() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[2].clade_not_completed(), 1);
}

#[test]
fn a_trial_not_completed_reaches_the_clade_not_completed_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn a_trial_not_completed_reaches_the_clade_not_completed_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 1);
}

#[test]
fn a_result_of_timed_out_reaches_the_clade_not_completed_of_the_primogenitor() {
    let genealogy = family_after(2, TrialResult::TimedOut);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 1);
}

#[test]
fn a_completed_trial_adds_nothing_to_the_clade_not_completed() {
    let genealogy = family_after(2, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 0);
}

#[test]
fn a_trial_not_completed_adds_nothing_to_the_clade_completed() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 0);
}

#[test]
fn a_completed_trial_stays_out_of_the_clade_completed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[3].clade_completed(), 0);
}

#[test]
fn a_completed_trial_stays_out_of_the_clade_completed_of_a_child() {
    let genealogy = family_after(1, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[2].clade_completed(), 0);
}

#[test]
fn a_trial_not_completed_stays_out_of_the_clade_not_completed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[3].clade_not_completed(), 0);
}

#[test]
fn a_trial_not_completed_stays_out_of_the_clade_not_completed_of_a_child() {
    let genealogy = family_after(1, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[2].clade_not_completed(), 0);
}

fn family_after_a_diversification_that_created_no_child(outcome: CreationOutcome) -> Genealogy {
    let mut genealogy = family_after(1, TrialResult::NotCompleted);
    let child = genealogy.start_diversification(1).unwrap();
    genealogy.end_diversification(child, outcome).unwrap();
    genealogy
}

#[test]
fn a_diversification_that_failed_adds_nothing_to_the_clade_not_completed_of_its_progenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn a_diversification_whose_child_did_not_build_adds_nothing_to_the_clade_not_completed_of_its_progenitor()
 {
    let genealogy =
        family_after_a_diversification_that_created_no_child(CreationOutcome::BuildFailed);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn a_diversification_that_timed_out_adds_nothing_to_the_clade_not_completed_of_its_progenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn a_diversification_that_left_the_progenitor_s_clone_unchanged_adds_nothing_to_the_clade_of_its_progenitor()
 {
    let genealogy =
        family_after_a_diversification_that_created_no_child(CreationOutcome::Unchanged);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn a_diversification_that_failed_adds_nothing_to_the_clade_not_completed_of_the_primogenitor() {
    let genealogy = family_after_a_diversification_that_created_no_child(CreationOutcome::Failed);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 1);
}

#[test]
fn the_clade_completed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_completed(), 2);
}

#[test]
fn the_clade_not_completed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 2);
}

#[test]
fn the_clade_of_the_primogenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 4);
}

#[test]
fn the_clade_completion_rate_is_the_share_completed() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, TrialResult::Completed);
    finish(&mut genealogy, 1, TrialResult::Completed);
    finish(&mut genealogy, 2, TrialResult::Completed);
    finish(&mut genealogy, 3, TrialResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), Some(0.75));
}

#[test]
fn the_clade_completion_rate_of_a_clade_with_nothing_completed_is_zero() {
    let genealogy = family_after(2, TrialResult::NotCompleted);
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
    let genealogy = family_after(3, TrialResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_completion_rate(), None);
}
