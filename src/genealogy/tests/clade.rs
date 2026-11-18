use crate::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};

fn family() -> Genealogy {
    let mut genealogy = Genealogy::new().holding_out(&["a".to_owned()]);
    for progenitor in [0, 1, 0] {
        let child = genealogy.start_procreation(progenitor).unwrap();
        genealogy
            .end_procreation(child, ProcreationOutcome::Created)
            .unwrap();
    }
    genealogy
}

fn finish(genealogy: &mut Genealogy, harness: HarnessId, result: AttemptResult) {
    genealogy.start_attempt(harness, "a").unwrap();
    genealogy.end_attempt(harness, "a", result).unwrap();
}

fn family_after(harness: HarnessId, result: AttemptResult) -> Genealogy {
    let mut genealogy = family();
    finish(&mut genealogy, harness, result);
    genealogy
}

fn family_with_every_harness_finishing(result: AttemptResult) -> Genealogy {
    let mut genealogy = family();
    for harness in 0..4 {
        finish(&mut genealogy, harness, result);
    }
    genealogy
}

#[test]
fn a_completed_attempt_reaches_the_clade_completed_of_its_own_harness() {
    let genealogy = family_after(2, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[2].clade_completed(), 1);
}

#[test]
fn a_completed_attempt_reaches_the_clade_completed_of_its_progenitor() {
    let genealogy = family_after(2, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_completed(), 1);
}

#[test]
fn a_completed_attempt_reaches_the_clade_completed_of_the_primogenitor() {
    let genealogy = family_after(2, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 1);
}

#[test]
fn an_attempt_not_completed_reaches_the_clade_not_completed_of_its_own_harness() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[2].clade_not_completed(), 1);
}

#[test]
fn an_attempt_not_completed_reaches_the_clade_not_completed_of_its_progenitor() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 1);
}

#[test]
fn an_attempt_not_completed_reaches_the_clade_not_completed_of_the_primogenitor() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 1);
}

#[test]
fn a_result_of_timed_out_reaches_the_clade_not_completed_of_the_primogenitor() {
    let genealogy = family_after(2, AttemptResult::TimedOut);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 1);
}

#[test]
fn a_completed_attempt_adds_nothing_to_the_clade_not_completed() {
    let genealogy = family_after(2, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_not_completed(), 0);
}

#[test]
fn an_attempt_not_completed_adds_nothing_to_the_clade_completed() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 0);
}

#[test]
fn a_completed_attempt_stays_out_of_the_clade_completed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[3].clade_completed(), 0);
}

#[test]
fn a_completed_attempt_stays_out_of_the_clade_completed_of_a_child() {
    let genealogy = family_after(1, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[2].clade_completed(), 0);
}

#[test]
fn an_attempt_not_completed_stays_out_of_the_clade_not_completed_of_a_sibling_of_its_progenitor() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[3].clade_not_completed(), 0);
}

#[test]
fn an_attempt_not_completed_stays_out_of_the_clade_not_completed_of_a_child() {
    let genealogy = family_after(1, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[2].clade_not_completed(), 0);
}

#[test]
fn the_clade_completed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_completed(), 2);
}

#[test]
fn the_clade_not_completed_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].clade_not_completed(), 2);
}

#[test]
fn the_clade_of_the_primogenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[0].clade_completed(), 4);
}

#[test]
fn the_clade_score_is_the_share_completed() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, AttemptResult::Completed);
    finish(&mut genealogy, 1, AttemptResult::Completed);
    finish(&mut genealogy, 2, AttemptResult::Completed);
    finish(&mut genealogy, 3, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[0].clade_score(), Some(0.75));
}

#[test]
fn the_clade_score_of_a_clade_with_nothing_completed_is_zero() {
    let genealogy = family_after(2, AttemptResult::NotCompleted);
    assert_eq!(genealogy.harnesses()[1].clade_score(), Some(0.0));
}

#[test]
fn the_clade_score_has_no_value_in_a_new_genealogy() {
    assert_eq!(Genealogy::new().harnesses()[0].clade_score(), None);
}

#[test]
fn the_clade_score_has_no_value_while_the_only_attempt_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].clade_score(), None);
}

#[test]
fn the_clade_score_has_no_value_for_a_clade_the_results_fall_outside() {
    let genealogy = family_after(3, AttemptResult::Completed);
    assert_eq!(genealogy.harnesses()[1].clade_score(), None);
}
