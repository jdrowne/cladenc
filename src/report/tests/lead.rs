use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy};
use crate::journal::Settings;
use crate::report::Report;
use crate::report::lead::{self, Lead, OnlyLeaderCompleted};
use serde_json::Value;

const SELECTION: &str = "benchmark/s1";

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: ["c1", "c2", "c3", "c4", "c5", "o1", "s1"]
                .map(str::to_owned)
                .to_vec(),
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 17.0,
        holdout_tasks: ["c1", "c2", "c3", "c4", "c5", "s1"]
            .map(|id| format!("benchmark/{id}"))
            .to_vec(),
        corroboration_tasks_percentage: 68.0,
        corroboration_tasks: ["c1", "c2", "c3", "c4", "c5"]
            .map(|id| format!("benchmark/{id}"))
            .to_vec(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn benchmarked(genealogy: &mut Genealogy, harness: u64, completed: &[&str]) {
    for task in settings().tasks() {
        let result = if completed.contains(&task.as_str()) {
            AttemptResult::Completed
        } else {
            AttemptResult::NotCompleted
        };
        genealogy.start_attempt(harness, &task).unwrap();
        genealogy.end_attempt(harness, &task, result).unwrap();
    }
}

fn evolved(primogenitor: &[&str], child: &[&str]) -> Genealogy {
    let mut genealogy = settings().genealogy();
    let made = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(made, CreationOutcome::Created)
        .unwrap();
    benchmarked(&mut genealogy, 0, primogenitor);
    benchmarked(&mut genealogy, made, child);
    genealogy
}

fn ended(genealogy: &Genealogy) -> Report {
    let best_harness = settings().artificial_selection().best_harness(genealogy);
    Report::of(&settings(), genealogy, &[], Some(best_harness))
}

fn report_of(primogenitor: &[&str], child: &[&str]) -> Report {
    ended(&evolved(primogenitor, child))
}

fn lead_of(primogenitor: &[&str], child: &[&str]) -> Lead {
    report_of(primogenitor, child).corroboration_lead.unwrap()
}

fn counts_of(primogenitor: &[&str], child: &[&str]) -> (OnlyLeaderCompleted, u64) {
    let lead = lead_of(primogenitor, child);
    (lead.only_leader_completed, lead.only_primogenitor_completed)
}

fn only_the_best_harness_completed(tasks: u64) -> OnlyLeaderCompleted {
    OnlyLeaderCompleted::OnlyBestHarnessCompleted(tasks)
}

fn object_of(primogenitor: &[&str], child: &[&str]) -> Value {
    serde_json::from_str(&report_of(primogenitor, child).json()).unwrap()
}

fn line_of(primogenitor: &[&str], child: &[&str]) -> String {
    report_of(primogenitor, child)
        .table()
        .lines()
        .nth(3)
        .unwrap()
        .to_owned()
}

#[test]
fn the_corroboration_tasks_that_only_the_best_harness_completed_are_counted() {
    assert_eq!(
        counts_of(
            &["benchmark/c1"],
            &[SELECTION, "benchmark/c1", "benchmark/c2", "benchmark/c3"]
        ),
        (only_the_best_harness_completed(2), 0)
    );
}

#[test]
fn the_corroboration_tasks_that_only_the_primogenitor_completed_are_counted() {
    assert_eq!(
        counts_of(
            &["benchmark/c1", "benchmark/c4", "benchmark/c5"],
            &[SELECTION, "benchmark/c1", "benchmark/c2"]
        ),
        (only_the_best_harness_completed(1), 2)
    );
}

#[test]
fn a_corroboration_task_that_both_harnesses_completed_is_left_out() {
    assert_eq!(
        counts_of(&["benchmark/c1"], &[SELECTION, "benchmark/c1"]),
        (only_the_best_harness_completed(0), 0)
    );
}

#[test]
fn a_corroboration_task_that_neither_harness_completed_is_left_out() {
    assert_eq!(
        counts_of(&[], &[SELECTION]),
        (only_the_best_harness_completed(0), 0)
    );
}

#[test]
fn a_task_that_is_not_held_out_is_no_part_of_the_lead() {
    assert_eq!(
        counts_of(&[], &[SELECTION, "benchmark/o1"]),
        (only_the_best_harness_completed(0), 0)
    );
}

#[test]
fn the_selection_task_that_made_the_harness_the_best_is_no_part_of_the_lead() {
    assert_eq!(
        counts_of(&[], &[SELECTION, "benchmark/c1"]),
        (only_the_best_harness_completed(1), 0)
    );
}

#[test]
fn where_the_best_harness_is_the_primogenitor_the_report_holds_no_lead() {
    assert_eq!(
        report_of(&[SELECTION], &["benchmark/c1"]).corroboration_lead,
        None
    );
}

#[test]
fn where_the_primogenitor_is_alone_the_report_holds_no_lead() {
    assert_eq!(ended(&settings().genealogy()).corroboration_lead, None);
}

#[test]
fn the_lead_is_that_of_the_best_harness_and_not_of_a_harness_that_completed_more_corroboration_tasks()
 {
    let mut genealogy = evolved(&[], &[SELECTION]);
    let another = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(another, CreationOutcome::Created)
        .unwrap();
    benchmarked(
        &mut genealogy,
        another,
        &["benchmark/c1", "benchmark/c2", "benchmark/c3"],
    );
    let lead = ended(&genealogy).corroboration_lead.unwrap();
    assert_eq!(
        lead.only_leader_completed,
        only_the_best_harness_completed(0)
    );
}

#[test]
fn where_no_corroboration_task_tells_the_two_harnesses_apart_the_likelihood_is_1() {
    assert_eq!(lead_of(&[], &[SELECTION]).likelihood_by_luck, 1.0);
}

#[test]
fn the_likelihood_of_the_lead_of_the_report_is_that_of_its_two_counts() {
    let lead = lead_of(
        &["benchmark/c5"],
        &[SELECTION, "benchmark/c1", "benchmark/c2"],
    );
    assert_eq!(lead.likelihood_by_luck, lead::likelihood_by_luck(2, 1));
}

#[test]
fn fourteen_heads_or_more_in_twenty_tosses_come_60460_times_in_1048576() {
    assert_eq!(lead::likelihood_by_luck(14, 6), 60_460.0 / 1_048_576.0);
}

#[test]
fn one_task_that_only_the_best_harness_completed_is_as_likely_as_not_by_luck() {
    assert_eq!(lead::likelihood_by_luck(1, 0), 0.5);
}

#[test]
fn three_tasks_that_only_the_best_harness_completed_come_by_luck_one_time_in_eight() {
    assert_eq!(lead::likelihood_by_luck(3, 0), 0.125);
}

#[test]
fn a_best_harness_that_is_behind_has_a_lead_that_luck_gives_more_often_than_not() {
    assert_eq!(lead::likelihood_by_luck(2, 5), 120.0 / 128.0);
}

#[test]
fn a_best_harness_that_completed_no_task_the_primogenitor_did_not_has_a_likelihood_of_1() {
    assert_eq!(lead::likelihood_by_luck(0, 3), 1.0);
}

#[test]
fn as_many_tasks_on_each_side_give_a_likelihood_just_above_a_half() {
    assert!((lead::likelihood_by_luck(2000, 2000) - 0.5063).abs() < 0.0001);
}

#[test]
fn a_lead_of_thousands_of_tasks_has_a_likelihood_that_is_next_to_nothing_and_not_below_0() {
    assert!((0.0..1e-100).contains(&lead::likelihood_by_luck(3000, 1000)));
}

#[test]
fn the_likelihood_is_never_above_1() {
    assert!((0..40).all(|only_primogenitor| lead::likelihood_by_luck(0, only_primogenitor) <= 1.0));
}

#[test]
fn the_line_above_the_table_gives_the_two_counts_and_the_likelihood_to_three_decimal_places() {
    assert_eq!(
        line_of(&[], &[SELECTION, "benchmark/c1", "benchmark/c2"]),
        "on the corroboration tasks: only the best harness completed 2, only the primogenitor \
         completed 0, and luck alone gives a lead at least as large with likelihood 0.250"
    );
}

#[test]
fn the_line_above_the_table_says_that_a_best_harness_that_is_the_primogenitor_is_compared_with_none()
 {
    assert_eq!(
        line_of(&[SELECTION], &[]),
        "on the corroboration tasks: the best harness is the primogenitor, so there are no two \
         harnesses to compare"
    );
}

#[test]
fn the_object_holds_the_tasks_that_only_the_best_harness_completed() {
    let object = object_of(&[], &[SELECTION, "benchmark/c1", "benchmark/c2"]);
    assert_eq!(
        object["corroboration_lead"]["only_best_harness_completed"].as_u64(),
        Some(2)
    );
}

#[test]
fn the_object_holds_the_tasks_that_only_the_primogenitor_completed() {
    let object = object_of(&["benchmark/c3"], &[SELECTION]);
    assert_eq!(
        object["corroboration_lead"]["only_primogenitor_completed"].as_u64(),
        Some(1)
    );
}

#[test]
fn the_object_holds_the_likelihood_as_it_is_counted_and_not_to_three_decimal_places() {
    let object = object_of(
        &[],
        &[SELECTION, "benchmark/c1", "benchmark/c2", "benchmark/c3"],
    );
    assert_eq!(
        object["corroboration_lead"]["likelihood_by_luck"].as_f64(),
        Some(lead::likelihood_by_luck(3, 0))
    );
}

#[test]
fn the_object_of_a_best_harness_that_is_the_primogenitor_holds_no_lead() {
    assert!(object_of(&[SELECTION], &[])["corroboration_lead"].is_null());
}

#[test]
fn as_many_tasks_on_each_side_are_as_likely_as_three_tosses_in_four() {
    assert_eq!(lead::likelihood_by_luck(1, 1), 0.75);
}

#[test]
fn a_likelihood_counted_from_thousands_of_tasks_is_a_number() {
    assert!(lead::likelihood_by_luck(1000, 3000) > 0.999);
}
