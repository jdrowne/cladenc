use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{CreationOutcome, Genealogy, HarnessState, TrialResult};
use crate::journal::{Seconds, Settings};
use crate::report::Report;
use crate::report::leader::Leader;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_diversifications: 800,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn trialled(genealogy: &mut Genealogy, harness: u64, task: &str, result: TrialResult) {
    genealogy.start_trial(harness, task).unwrap();
    genealogy.end_trial(harness, task, result).unwrap();
}

fn a_child_that_succeeds_where_its_progenitor_fails() -> Genealogy {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    trialled(&mut genealogy, 0, "benchmark/t1", TrialResult::NotCompleted);
    trialled(&mut genealogy, 0, "benchmark/t2", TrialResult::Completed);
    trialled(&mut genealogy, 0, "benchmark/t3", TrialResult::Completed);
    trialled(
        &mut genealogy,
        child,
        "benchmark/t1",
        TrialResult::Completed,
    );
    trialled(
        &mut genealogy,
        child,
        "benchmark/t2",
        TrialResult::NotCompleted,
    );
    trialled(
        &mut genealogy,
        child,
        "benchmark/t3",
        TrialResult::NotCompleted,
    );
    genealogy
}

fn a_child_that_is_being_created_and_one_that_failed() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let failed = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(failed, CreationOutcome::Failed)
        .unwrap();
    genealogy.start_diversification(0).unwrap();
    genealogy.start_trial(0, "benchmark/t1").unwrap();
    genealogy
}

fn seconds() -> [Seconds; 2] {
    [
        Seconds {
            trials: 0.1 + 0.2,
            diversification: None,
        },
        Seconds {
            trials: 7.5,
            diversification: Some(30.25),
        },
    ]
}

fn report(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy, &seconds(), None)
}

#[test]
fn a_clade_with_no_finished_trial_has_no_clade_score() {
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
fn the_selection_counts_of_a_harness_are_those_of_its_own_trials_on_the_selection_tasks() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].selection_completed,
            report.harnesses[0].selection_not_completed
        ),
        (0, 1)
    );
}

#[test]
fn the_corroboration_counts_of_a_harness_are_those_of_its_own_trials_on_the_corroboration_tasks() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].corroboration_completed,
            report.harnesses[0].corroboration_not_completed
        ),
        (1, 0)
    );
}

#[test]
fn the_corroboration_counts_of_a_child_are_its_own_and_not_those_of_its_progenitor() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[1].corroboration_completed,
            report.harnesses[1].corroboration_not_completed
        ),
        (0, 1)
    );
}

#[test]
fn the_non_holdout_counts_of_a_harness_are_those_of_its_own_trials_on_the_non_holdout_tasks() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.harnesses[0].non_holdout_completed,
            report.harnesses[0].non_holdout_not_completed
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
fn the_harness_that_is_ahead_so_far_is_the_one_the_artificial_selection_would_name_the_best() {
    let genealogy = a_child_that_succeeds_where_its_progenitor_fails();
    assert_eq!(
        report(&genealogy).leader,
        Leader::HarnessAheadSoFar(settings().artificial_selection().best_harness(&genealogy))
    );
}

#[test]
fn the_harness_that_is_ahead_so_far_is_benchmarked_in_full_and_completed_the_most_selection_tasks()
{
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.leader, Leader::HarnessAheadSoFar(1));
}

#[test]
fn the_best_harness_of_a_directed_evolution_that_has_ended_is_the_one_its_journal_names() {
    let genealogy = a_child_that_succeeds_where_its_progenitor_fails();
    let report = Report::of(&settings(), &genealogy, &seconds(), Some(0));
    assert_eq!(report.leader, Leader::BestHarness(0));
}

#[test]
fn a_report_made_with_a_best_harness_is_of_a_directed_evolution_that_has_ended() {
    let report = Report::of(&settings(), &Genealogy::new(), &seconds(), Some(0));
    assert!(report.has_ended());
}

#[test]
fn a_report_made_with_no_best_harness_is_of_a_directed_evolution_that_has_not_ended() {
    assert!(!report(&Genealogy::new()).has_ended());
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
fn the_diversifications_started_count_one_that_failed_and_one_that_has_not_ended() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(report.diversifications_started, 2);
}

#[test]
fn no_diversification_has_started_where_the_primogenitor_is_alone() {
    let report = report(&Genealogy::new());
    assert_eq!(report.diversifications_started, 0);
}

#[test]
fn the_most_diversifications_is_that_of_the_run() {
    let report = report(&Genealogy::new());
    assert_eq!(report.max_diversifications, 800);
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
fn the_progenitor_of_a_child_is_the_harness_it_is_a_clone_of() {
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
fn a_directed_evolution_whose_primogenitor_awaits_its_build_has_harness_0_ahead_so_far() {
    assert_eq!(
        report(&Genealogy::unbuilt()).leader,
        Leader::HarnessAheadSoFar(0)
    );
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
fn the_seconds_of_the_trials_of_a_harness_are_reported_to_the_millisecond() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[0].trial_seconds, 0.3);
}

#[test]
fn the_seconds_of_the_diversification_that_made_a_harness_are_reported() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[1].diversification_seconds, Some(30.25));
}

#[test]
fn the_primogenitor_was_made_by_no_diversification_and_has_no_seconds_of_one() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(report.harnesses[0].diversification_seconds, None);
}

#[test]
fn a_harness_for_which_no_seconds_are_given_is_reported_with_none_spent() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        (
            report.harnesses[2].trial_seconds,
            report.harnesses[2].diversification_seconds
        ),
        (0.0, None)
    );
}

#[test]
fn the_selection_tasks_that_at_least_one_harness_completed_are_counted_out_of_all_of_them() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.selection_tasks_completed_by_any,
            report.selection_tasks
        ),
        (1, 1)
    );
}

#[test]
fn the_corroboration_tasks_that_at_least_one_harness_completed_are_counted_out_of_all_of_them() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.corroboration_tasks_completed_by_any,
            report.corroboration_tasks
        ),
        (1, 1)
    );
}

#[test]
fn the_tasks_of_each_set_are_counted_where_no_harness_has_completed_one() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        (
            report.selection_tasks,
            report.corroboration_tasks,
            report.non_holdout_tasks
        ),
        (1, 1, 1)
    );
}

#[test]
fn the_non_holdout_tasks_that_at_least_one_harness_completed_are_counted_out_of_all_of_them() {
    let report = report(&a_child_that_succeeds_where_its_progenitor_fails());
    assert_eq!(
        (
            report.non_holdout_tasks_completed_by_any,
            report.non_holdout_tasks
        ),
        (1, 1)
    );
}

#[test]
fn a_task_that_no_harness_completed_is_not_counted_as_completed_by_one_at_least() {
    let report = report(&a_child_that_is_being_created_and_one_that_failed());
    assert_eq!(
        (
            report.selection_tasks_completed_by_any,
            report.corroboration_tasks_completed_by_any,
            report.non_holdout_tasks_completed_by_any
        ),
        (0, 0, 0)
    );
}

#[test]
fn a_task_that_two_harnesses_completed_is_counted_once() {
    let mut genealogy = a_child_that_succeeds_where_its_progenitor_fails();
    let another = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(another, CreationOutcome::Created)
        .unwrap();
    trialled(
        &mut genealogy,
        another,
        "benchmark/t1",
        TrialResult::Completed,
    );
    assert_eq!(report(&genealogy).selection_tasks_completed_by_any, 1);
}
