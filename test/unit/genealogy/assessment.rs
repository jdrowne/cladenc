use cladenc::genealogy::{AssessmentOutcome, Genealogy, ProcreationOutcome};

fn assessed(outcome: AssessmentOutcome) -> Genealogy {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    genealogy.end_assessment(0, "a", outcome).unwrap();
    genealogy
}

#[test]
fn started_counts_an_assessment_that_has_not_ended() {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    assert_eq!(genealogy.started(), 1);
}

#[test]
fn started_keeps_counting_an_assessment_that_has_ended() {
    assert_eq!(assessed(AssessmentOutcome::Success).started(), 1);
}

#[test]
fn started_counts_the_assessments_of_every_harness() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy.start_assessment(0, "a").unwrap();
    genealogy.start_assessment(0, "b").unwrap();
    genealogy.start_assessment(child, "a").unwrap();
    assert_eq!(genealogy.started(), 3);
}

#[test]
fn an_assessment_that_has_not_ended_counts_as_neither_success_nor_failure() {
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    let progenitor = &genealogy.harnesses()[0];
    assert_eq!((progenitor.successes(), progenitor.failures()), (0, 0));
}

#[test]
fn a_success_adds_one_to_the_successes_of_its_harness() {
    let genealogy = assessed(AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].successes(), 1);
}

#[test]
fn a_success_adds_nothing_to_the_failures_of_its_harness() {
    let genealogy = assessed(AssessmentOutcome::Success);
    assert_eq!(genealogy.harnesses()[0].failures(), 0);
}

#[test]
fn a_failure_adds_one_to_the_failures_of_its_harness() {
    let genealogy = assessed(AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].failures(), 1);
}

#[test]
fn a_failure_adds_nothing_to_the_successes_of_its_harness() {
    let genealogy = assessed(AssessmentOutcome::Failure);
    assert_eq!(genealogy.harnesses()[0].successes(), 0);
}

#[test]
fn an_assessment_that_timed_out_counts_as_a_failure() {
    let genealogy = assessed(AssessmentOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].failures(), 1);
}

#[test]
fn an_assessment_that_timed_out_adds_nothing_to_the_successes() {
    let genealogy = assessed(AssessmentOutcome::TimedOut);
    assert_eq!(genealogy.harnesses()[0].successes(), 0);
}

#[test]
fn the_results_of_a_harness_add_up_over_its_tasks() {
    let mut genealogy = assessed(AssessmentOutcome::Success);
    genealogy.start_assessment(0, "b").unwrap();
    genealogy
        .end_assessment(0, "b", AssessmentOutcome::Success)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].successes(), 2);
}
