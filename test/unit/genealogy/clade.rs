use cladenc::genealogy::{Genealogy, HarnessId, ProcreationOutcome, Score};

fn family() -> Genealogy {
    let mut genealogy = Genealogy::new();
    for parent in [0, 1, 0] {
        let child = genealogy.start_procreation(parent).unwrap();
        genealogy
            .end_procreation(child, ProcreationOutcome::Created)
            .unwrap();
    }
    genealogy
}

fn finish(genealogy: &mut Genealogy, harness: HarnessId, score: Score) {
    genealogy.start_assessment(harness, "a").unwrap();
    genealogy.end_assessment(harness, "a", score).unwrap();
}

fn family_after(harness: HarnessId, score: Score) -> Genealogy {
    let mut genealogy = family();
    finish(&mut genealogy, harness, score);
    genealogy
}

fn family_with_every_harness_finishing(score: Score) -> Genealogy {
    let mut genealogy = family();
    for harness in 0..4 {
        finish(&mut genealogy, harness, score);
    }
    genealogy
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_own_harness() {
    let genealogy = family_after(2, Score::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_parent() {
    let genealogy = family_after(2, Score::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_the_primogenitor() {
    let genealogy = family_after(2, Score::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_own_harness() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_parent() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_the_primogenitor() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn a_score_of_timed_out_reaches_the_clade_failures_of_the_primogenitor() {
    let genealogy = family_after(2, Score::TimedOut);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn a_success_adds_nothing_to_the_clade_failures() {
    let genealogy = family_after(2, Score::Success);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 0);
}

#[test]
fn a_failure_adds_nothing_to_the_clade_successes() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, Score::Success);
    assert_eq!(genealogy.harnesses()[3].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_child() {
    let genealogy = family_after(1, Score::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[3].clade_failures(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_child() {
    let genealogy = family_after(1, Score::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 0);
}

#[test]
fn the_clade_successes_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(Score::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 2);
}

#[test]
fn the_clade_failures_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(Score::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 2);
}

#[test]
fn the_clade_of_the_primogenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(Score::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 4);
}

#[test]
fn the_clade_success_rate_is_the_share_of_successes() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, Score::Success);
    finish(&mut genealogy, 1, Score::Success);
    finish(&mut genealogy, 2, Score::Success);
    finish(&mut genealogy, 3, Score::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_success_rate(), Some(0.75));
}

#[test]
fn the_clade_success_rate_of_a_clade_of_failures_is_zero() {
    let genealogy = family_after(2, Score::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_success_rate(), Some(0.0));
}

#[test]
fn the_clade_success_rate_has_no_value_in_a_new_genealogy() {
    assert_eq!(Genealogy::new().harnesses()[0].clade_success_rate(), None);
}

#[test]
fn the_clade_success_rate_has_no_value_while_the_only_assessment_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].clade_success_rate(), None);
}

#[test]
fn the_clade_success_rate_has_no_value_for_a_clade_the_results_fall_outside() {
    let genealogy = family_after(3, Score::Success);
    assert_eq!(genealogy.harnesses()[1].clade_success_rate(), None);
}
