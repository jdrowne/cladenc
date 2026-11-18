use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, Genealogy, HarnessState, ProcreationOutcome};
use crate::journal::{Seconds, Settings};
use crate::report::Report;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        genealogy_growth: 0.6,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        attempt_timeout: 60,
        holdout_percentage: 50.0,
        holdout_tasks: vec!["benchmark/t1".to_owned()],
        seed: 7,
    }
}

fn attempted(genealogy: &mut Genealogy, harness: u64, task: &str, result: AttemptResult) {
    genealogy.start_attempt(harness, task).unwrap();
    genealogy.end_attempt(harness, task, result).unwrap();
}

fn a_child_that_succeeds_where_its_progenitor_fails() -> Genealogy {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    attempted(
        &mut genealogy,
        0,
        "benchmark/t1",
        AttemptResult::NotCompleted,
    );
    attempted(&mut genealogy, 0, "benchmark/t2", AttemptResult::Completed);
    attempted(
        &mut genealogy,
        child,
        "benchmark/t1",
        AttemptResult::Completed,
    );
    attempted(
        &mut genealogy,
        child,
        "benchmark/t2",
        AttemptResult::NotCompleted,
    );
    genealogy
}

fn a_child_that_is_being_created_and_one_that_failed() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let failed = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(failed, ProcreationOutcome::Failed)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    genealogy.start_attempt(0, "benchmark/t1").unwrap();
    genealogy
}

fn seconds() -> [Seconds; 2] {
    [
        Seconds {
            attempts: 0.1 + 0.2,
            procreation: None,
        },
        Seconds {
            attempts: 7.5,
            procreation: Some(30.25),
        },
    ]
}

fn report(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy, &seconds())
}

#[test]
fn a_clade_with_no_finished_attempt_has_no_clade_score() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.harnesses[0].clade_score, None);
}

#[test]
fn the_clade_score_of_a_harness_counts_its_descendants() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[0].clade_score, Some(0.5));
}

#[test]
fn the_clade_counts_of_a_harness_count_its_descendants() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].clade_completed,
            report.harnesses[0].clade_not_completed
        ),
        (1, 1)
    );
}

#[test]
fn the_holdout_counts_of_a_harness_are_those_of_its_own_attempts_at_the_holdout_tasks() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].holdout_completed,
            report.harnesses[0].holdout_not_completed
        ),
        (0, 1)
    );
}

#[test]
fn the_other_counts_of_a_harness_are_those_of_its_own_attempts_at_the_other_tasks() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].other_completed,
            report.harnesses[0].other_not_completed
        ),
        (1, 0)
    );
}

#[test]
fn the_clade_counts_of_a_harness_leave_out_the_tasks_that_are_not_held_out() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[1].clade_completed,
            report.harnesses[1].clade_not_completed
        ),
        (1, 0)
    );
}

#[test]
fn the_best_harness_reported_is_the_one_the_artificial_selection_chooses() {
    let genealogy = a_child_that_succeeds_where_its_progenitor_fails();
    assert_eq!(
        report(&genealogy).best_harness,
        settings().artificial_selection().best_harness(&genealogy)
    );
}

#[test]
fn the_best_harness_reported_is_the_one_benchmarked_in_full_that_completed_the_most_holdout_tasks()
{
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.best_harness, 1);
}

#[test]
fn the_model_reported_is_the_one_the_directed_evolution_was_started_with() {
    let report = report(&Genealogy::new());
    assert_eq!(
        (report.model.as_str(), report.model_url.as_str()),
        ("a-model", "http://models.example/v1")
    );
}

#[test]
fn the_procreations_started_count_one_that_failed_and_one_that_has_not_ended() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.procreations_started, 2);
}

#[test]
fn no_procreation_has_started_where_the_primogenitor_is_alone() {
    let report = report(&Genealogy::new());
    assert_eq!(report.procreations_started, 0);
}

#[test]
fn the_most_procreations_is_that_of_the_run() {
    let report = report(&Genealogy::new());
    assert_eq!(report.max_procreations, 800);
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
fn the_primogenitor_has_no_progenitor() {
    let report = report(&Genealogy::new());
    assert_eq!(report.harnesses[0].progenitor, None);
}

#[test]
fn the_progenitor_of_a_child_is_the_harness_that_procreated_it() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[1].progenitor, Some(0));
}

#[test]
fn a_primogenitor_that_awaits_its_build_is_reported_as_being_created() {
    assert_eq!(
        report(&Genealogy::unbuilt()).harnesses[0].state,
        HarnessState::BeingCreated
    );
}

#[test]
fn a_directed_evolution_whose_primogenitor_awaits_its_build_reports_harness_0_as_its_best() {
    assert_eq!(report(&Genealogy::unbuilt()).best_harness, 0);
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

#[test]
fn the_seconds_of_the_attempts_of_a_harness_are_reported_to_the_millisecond() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[0].attempt_seconds, 0.3);
}

#[test]
fn the_seconds_of_the_procreation_that_made_a_harness_are_reported() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[1].procreation_seconds, Some(30.25));
}

#[test]
fn the_primogenitor_was_made_by_no_procreation_and_has_no_seconds_of_one() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[0].procreation_seconds, None);
}

#[test]
fn a_harness_for_which_no_seconds_are_given_is_reported_with_none_spent() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        (
            report.harnesses[2].attempt_seconds,
            report.harnesses[2].procreation_seconds
        ),
        (0.0, None)
    );
}

#[test]
fn the_holdout_tasks_that_at_least_one_harness_completed_are_counted_out_of_all_of_them() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (report.holdout_tasks_completed_by_any, report.holdout_tasks),
        (1, 1)
    );
}

#[test]
fn the_other_tasks_that_at_least_one_harness_completed_are_counted_out_of_all_of_them() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (report.other_tasks_completed_by_any, report.other_tasks),
        (1, 1)
    );
}

#[test]
fn a_task_that_no_harness_completed_is_not_counted_as_completed_by_one_at_least() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        (
            report.holdout_tasks_completed_by_any,
            report.other_tasks_completed_by_any
        ),
        (0, 0)
    );
}

#[test]
fn a_task_that_two_harnesses_completed_is_counted_once() {
    let mut genealogy = a_child_that_succeeds_where_its_progenitor_fails();
    let another = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(another, ProcreationOutcome::Created)
        .unwrap();
    attempted(
        &mut genealogy,
        another,
        "benchmark/t1",
        AttemptResult::Completed,
    );
    assert_eq!(report(&genealogy).holdout_tasks_completed_by_any, 1);
}
