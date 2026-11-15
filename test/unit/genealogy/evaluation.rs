use cladenc::genealogy::{EvaluationOutcome, ExpansionOutcome, Genealogy};

fn evaluated(outcome: EvaluationOutcome) -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    genealogy.end_evaluation(0, "a", outcome).unwrap();
    genealogy
}

#[test]
fn started_counts_an_evaluation_that_has_not_ended() {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    assert_eq!(genealogy.started(), 1);
}

#[test]
fn started_keeps_counting_an_evaluation_that_has_ended() {
    assert_eq!(evaluated(EvaluationOutcome::Success).started(), 1);
}

#[test]
fn started_counts_the_evaluations_of_every_harness() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_expansion(0).unwrap();
    genealogy
        .end_expansion(child, ExpansionOutcome::Created)
        .unwrap();
    genealogy.start_evaluation(0, "a").unwrap();
    genealogy.start_evaluation(0, "b").unwrap();
    genealogy.start_evaluation(child, "a").unwrap();
    assert_eq!(genealogy.started(), 3);
}

#[test]
fn an_evaluation_that_has_not_ended_counts_as_neither_success_nor_failure() {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    let progenitor = &genealogy.harnesses()[0];
    assert_eq!((progenitor.successes(), progenitor.failures()), (0, 0));
}

#[test]
fn a_success_adds_one_to_the_successes_of_its_harness() {
    let genealogy = evaluated(EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].successes(), 1);
}

#[test]
fn a_success_adds_nothing_to_the_failures_of_its_harness() {
    let genealogy = evaluated(EvaluationOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].failures(), 0);
}

#[test]
fn a_failure_adds_one_to_the_failures_of_its_harness() {
    let genealogy = evaluated(EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].failures(), 1);
}

#[test]
fn a_failure_adds_nothing_to_the_successes_of_its_harness() {
    let genealogy = evaluated(EvaluationOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].successes(), 0);
}

#[test]
fn an_evaluation_that_timed_out_counts_as_a_failure() {
    let genealogy = evaluated(EvaluationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].failures(), 1);
}

#[test]
fn an_evaluation_that_timed_out_adds_nothing_to_the_successes() {
    let genealogy = evaluated(EvaluationOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].successes(), 0);
}

#[test]
fn the_results_of_a_harness_add_up_over_its_tasks() {
    let mut genealogy = evaluated(EvaluationOutcome::Success);
    genealogy.start_evaluation(0, "b").unwrap();
    genealogy
        .end_evaluation(0, "b", EvaluationOutcome::Success)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].successes(), 2);
}
