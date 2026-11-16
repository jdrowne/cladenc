use cladenc::genealogy::{AssessmentOutcome, Genealogy, HarnessId, ProcreationOutcome};

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

fn finish(genealogy: &mut Genealogy, harness: HarnessId, outcome: AssessmentOutcome) {
    genealogy.start_assessment(harness, "a").unwrap();
    genealogy.end_assessment(harness, "a", outcome).unwrap();
}

fn family_after(harness: HarnessId, outcome: AssessmentOutcome) -> Genealogy {
    let mut genealogy = family();
    finish(&mut genealogy, harness, outcome);
    genealogy
}

fn family_with_every_harness_finishing(outcome: AssessmentOutcome) -> Genealogy {
    let mut genealogy = family();
    for harness in 0..4 {
        finish(&mut genealogy, harness, outcome);
    }
    genealogy
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_own_harness() {
    let genealogy = family_after(2, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_parent() {
    let genealogy = family_after(2, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_the_progenitor() {
    let genealogy = family_after(2, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_own_harness() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_parent() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_the_progenitor() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn an_assessment_that_timed_out_reaches_the_clade_failures_of_the_progenitor() {
    let genealogy = family_after(2, AssessmentOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn a_success_adds_nothing_to_the_clade_failures() {
    let genealogy = family_after(2, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 0);
}

#[test]
fn a_failure_adds_nothing_to_the_clade_successes() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[3].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_child() {
    let genealogy = family_after(1, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[3].clade_failures(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_child() {
    let genealogy = family_after(1, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 0);
}

#[test]
fn the_clade_successes_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 2);
}

#[test]
fn the_clade_failures_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 2);
}

#[test]
fn the_clade_of_the_progenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 4);
}

#[test]
fn the_clade_success_rate_is_the_share_of_successes() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, AssessmentOutcome::Success);
    finish(&mut genealogy, 1, AssessmentOutcome::Success);
    finish(&mut genealogy, 2, AssessmentOutcome::Success);
    finish(&mut genealogy, 3, AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_success_rate(), Some(0.75));
}

#[test]
fn the_clade_success_rate_of_a_clade_of_failures_is_zero() {
    let genealogy = family_after(2, AssessmentOutcome::Failure);
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
    let genealogy = family_after(3, AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_success_rate(), None);
}
