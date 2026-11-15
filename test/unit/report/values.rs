use std::path::PathBuf;

use cladenc::genealogy::{EvaluationOutcome, Genealogy, HarnessState, ProcreationOutcome};
use cladenc::journal::Settings;
use cladenc::policy::lower_bound;
use cladenc::report::Report;

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/progenitor"),
        tasks: vec!["t1".to_owned(), "t2".to_owned()],
        procreator: PathBuf::from("/procreator"),
        evaluator: PathBuf::from("/evaluator"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 0,
        workers: 1,
        procreation_timeout: 60,
        evaluation_timeout: 60,
        seed: 7,
    }
}

fn a_child_that_succeeds_where_its_parent_fails() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy.start_evaluation(0, "t1").unwrap();
    genealogy
        .end_evaluation(0, "t1", EvaluationOutcome::Failure)
        .unwrap();
    genealogy.start_evaluation(child, "t1").unwrap();
    genealogy
        .end_evaluation(child, "t1", EvaluationOutcome::Success)
        .unwrap();
    genealogy
}

fn a_child_that_is_being_created_and_one_that_failed() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let failed = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(failed, ProcreationOutcome::Failed)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    genealogy.start_evaluation(0, "t1").unwrap();
    genealogy
}

fn report(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy)
}

#[test]
fn a_created_harness_has_the_lower_bound_of_its_own_results_at_the_confidence_of_the_run() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(
        report.harnesses[1].lower_bound,
        Some(lower_bound(1, 0, 99.0))
    );
}

#[test]
fn a_failed_harness_has_no_lower_bound() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.harnesses[1].lower_bound, None);
}

#[test]
fn a_harness_that_is_being_created_has_no_lower_bound() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.harnesses[2].lower_bound, None);
}

#[test]
fn a_clade_with_no_finished_evaluation_has_no_clade_success_rate() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.harnesses[0].clade_success_rate, None);
}

#[test]
fn the_clade_success_rate_of_a_harness_counts_its_descendants() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(report.harnesses[0].clade_success_rate, Some(0.5));
}

#[test]
fn the_clade_counts_of_a_harness_count_its_descendants() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(
        (
            report.harnesses[0].clade_successes,
            report.harnesses[0].clade_failures
        ),
        (1, 1)
    );
}

#[test]
fn the_counts_of_a_harness_are_those_of_its_own_evaluations() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(
        (report.harnesses[0].successes, report.harnesses[0].failures),
        (0, 1)
    );
}

#[test]
fn the_best_harness_reported_is_the_one_the_policy_chooses() {
    let genealogy = a_child_that_succeeds_where_its_parent_fails();
    assert_eq!(
        report(&genealogy).best_harness,
        settings().policy().best_harness(&genealogy)
    );
}

#[test]
fn the_best_harness_reported_is_the_created_harness_with_the_largest_lower_bound() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(report.best_harness, 1);
}

#[test]
fn started_counts_an_evaluation_that_has_not_finished() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.started, 1);
}

#[test]
fn the_budget_is_that_of_the_run() {
    let report = report(&Genealogy::new());
    assert_eq!(report.budget, 800);
}

#[test]
fn the_harnesses_are_listed_in_ascending_id_order() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        report
            .harnesses
            .iter()
            .map(|harness| harness.id)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

#[test]
fn the_progenitor_has_no_parent() {
    let report = report(&Genealogy::new());
    assert_eq!(report.harnesses[0].parent, None);
}

#[test]
fn the_parent_of_a_child_is_the_harness_that_procreated_it() {
    let report = report(&a_child_that_succeeds_where_its_parent_fails());
    assert_eq!(report.harnesses[1].parent, Some(0));
}

#[test]
fn the_state_of_each_harness_is_reported() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        report
            .harnesses
            .iter()
            .map(|harness| harness.state)
            .collect::<Vec<_>>(),
        [
            HarnessState::Created,
            HarnessState::Failed,
            HarnessState::BeingCreated
        ]
    );
}
