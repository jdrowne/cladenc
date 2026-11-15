use cladenc::genealogy::{EvaluationOutcome, Genealogy, HarnessId, MutationOutcome};

fn family() -> Genealogy {
    let mut genealogy = Genealogy::new();
    for parent in [0, 1, 0] {
        let child = genealogy.start_mutation(parent).unwrap();
        genealogy
            .end_mutation(child, MutationOutcome::Created)
            .unwrap();
    }
    genealogy
}

fn finish(genealogy: &mut Genealogy, harness: HarnessId, outcome: EvaluationOutcome) {
    genealogy.start_evaluation(harness, "a").unwrap();
    genealogy.end_evaluation(harness, "a", outcome).unwrap();
}

fn family_after(harness: HarnessId, outcome: EvaluationOutcome) -> Genealogy {
    let mut genealogy = family();
    finish(&mut genealogy, harness, outcome);
    genealogy
}

fn family_with_every_harness_finishing(outcome: EvaluationOutcome) -> Genealogy {
    let mut genealogy = family();
    for harness in 0..4 {
        finish(&mut genealogy, harness, outcome);
    }
    genealogy
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_own_harness() {
    let genealogy = family_after(2, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_its_parent() {
    let genealogy = family_after(2, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 1);
}

#[test]
fn a_success_reaches_the_clade_successes_of_the_progenitor() {
    let genealogy = family_after(2, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_own_harness() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_its_parent() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 1);
}

#[test]
fn a_failure_reaches_the_clade_failures_of_the_progenitor() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn an_evaluation_that_timed_out_reaches_the_clade_failures_of_the_progenitor() {
    let genealogy = family_after(2, EvaluationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 1);
}

#[test]
fn a_success_adds_nothing_to_the_clade_failures() {
    let genealogy = family_after(2, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_failures(), 0);
}

#[test]
fn a_failure_adds_nothing_to_the_clade_successes() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[3].clade_successes(), 0);
}

#[test]
fn a_success_stays_out_of_the_clade_successes_of_a_child() {
    let genealogy = family_after(1, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[2].clade_successes(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_sibling_of_its_parent() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[3].clade_failures(), 0);
}

#[test]
fn a_failure_stays_out_of_the_clade_failures_of_a_child() {
    let genealogy = family_after(1, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[2].clade_failures(), 0);
}

#[test]
fn the_clade_successes_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_successes(), 2);
}

#[test]
fn the_clade_failures_are_summed_over_the_clade() {
    let genealogy = family_with_every_harness_finishing(EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_failures(), 2);
}

#[test]
fn the_clade_of_the_progenitor_is_the_whole_genealogy() {
    let genealogy = family_with_every_harness_finishing(EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].clade_successes(), 4);
}

#[test]
fn the_clade_success_rate_is_the_share_of_successes() {
    let mut genealogy = family();
    finish(&mut genealogy, 0, EvaluationOutcome::Success);
    finish(&mut genealogy, 1, EvaluationOutcome::Success);
    finish(&mut genealogy, 2, EvaluationOutcome::Success);
    finish(&mut genealogy, 3, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].clade_success_rate(), Some(0.75));
}

#[test]
fn the_clade_success_rate_of_a_clade_of_failures_is_zero() {
    let genealogy = family_after(2, EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[1].clade_success_rate(), Some(0.0));
}

#[test]
fn the_clade_success_rate_has_no_value_in_a_new_genealogy() {
    assert_eq!(Genealogy::new().harnesses()[0].clade_success_rate(), None);
}

#[test]
fn the_clade_success_rate_has_no_value_while_the_only_evaluation_runs() {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].clade_success_rate(), None);
}

#[test]
fn the_clade_success_rate_has_no_value_for_a_clade_the_results_fall_outside() {
    let genealogy = family_after(3, EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[1].clade_success_rate(), None);
}
