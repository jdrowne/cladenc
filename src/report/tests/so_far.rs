use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy};
use crate::journal::Settings;
use crate::report::Report;
use crate::report::lead::OnlyLeaderCompleted;
use serde_json::Value;

const EVERY_HOLDOUT_TASK: [&str; 3] = ["benchmark/c1", "benchmark/c2", "benchmark/s1"];

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: ["c1", "c2", "o1", "s1"].map(str::to_owned).to_vec(),
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        attempt_timeout: 60,
        selection_tasks_percentage: 25.5,
        holdout_tasks: EVERY_HOLDOUT_TASK.map(str::to_owned).to_vec(),
        corroboration_tasks_percentage: 49.5,
        corroboration_tasks: vec!["benchmark/c1".to_owned(), "benchmark/c2".to_owned()],
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

fn with_a_child_ahead() -> Genealogy {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    benchmarked(&mut genealogy, 0, &[]);
    benchmarked(&mut genealogy, child, &EVERY_HOLDOUT_TASK);
    genealogy
}

fn so_far(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy, &[], None)
}

fn ended(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy, &[], Some(1))
}

fn line_of(report: &Report, line: usize) -> String {
    report.table().lines().nth(line).unwrap().to_owned()
}

fn object_of(report: &Report) -> Value {
    serde_json::from_str(&report.json()).unwrap()
}

#[test]
fn the_table_of_a_directed_evolution_that_has_not_ended_says_so_first() {
    assert_eq!(
        line_of(&so_far(&with_a_child_ahead()), 0),
        "the directed evolution has not ended: what follows is its state so far"
    );
}

#[test]
fn the_table_of_a_directed_evolution_that_has_ended_starts_with_the_model() {
    assert_eq!(
        line_of(&ended(&with_a_child_ahead()), 0),
        "model a-model at http://models.example/v1"
    );
}

#[test]
fn the_model_follows_the_line_that_says_that_the_directed_evolution_has_not_ended() {
    assert_eq!(
        line_of(&so_far(&with_a_child_ahead()), 1),
        "model a-model at http://models.example/v1"
    );
}

#[test]
fn the_table_of_a_directed_evolution_that_has_not_ended_names_the_harness_that_is_ahead_so_far() {
    assert_eq!(
        line_of(&so_far(&with_a_child_ahead()), 2),
        "started 1 of 800 procreations, harness ahead so far 1"
    );
}

#[test]
fn the_table_of_a_directed_evolution_that_has_ended_names_the_best_harness() {
    assert_eq!(
        line_of(&ended(&with_a_child_ahead()), 1),
        "started 1 of 800 procreations, best harness 1"
    );
}

#[test]
fn the_lead_of_the_harness_that_is_ahead_so_far_is_said_in_the_words_of_the_state_so_far() {
    assert_eq!(
        line_of(&so_far(&with_a_child_ahead()), 4),
        "on the corroboration tasks: only the harness that is ahead so far completed 2, only the \
         primogenitor completed 0, and luck alone gives a lead at least as large with likelihood \
         0.250"
    );
}

#[test]
fn a_harness_ahead_so_far_that_is_the_primogenitor_is_said_to_be_compared_with_none() {
    assert_eq!(
        line_of(&so_far(&settings().genealogy()), 4),
        "on the corroboration tasks: the harness that is ahead so far is the primogenitor, so \
         there are no two harnesses to compare"
    );
}

#[test]
fn the_table_of_a_directed_evolution_that_has_not_ended_never_says_best_harness() {
    assert!(
        !so_far(&with_a_child_ahead())
            .table()
            .contains("best harness")
    );
}

#[test]
fn the_table_of_a_directed_evolution_that_has_not_ended_holds_a_row_for_each_harness() {
    assert!(line_of(&so_far(&with_a_child_ahead()), 10).starts_with("1   0           -  "));
}

#[test]
fn the_object_of_a_directed_evolution_that_has_not_ended_says_so_first() {
    assert!(
        so_far(&with_a_child_ahead())
            .json()
            .starts_with(r#"{"ended":false,"model":"a-model","#)
    );
}

#[test]
fn the_object_of_a_directed_evolution_that_has_ended_says_so_first() {
    assert!(
        ended(&with_a_child_ahead())
            .json()
            .starts_with(r#"{"ended":true,"model":"a-model","#)
    );
}

#[test]
fn the_object_names_the_harness_that_is_ahead_so_far_where_a_report_names_the_best_harness() {
    assert!(so_far(&with_a_child_ahead()).json().contains(
        r#""max_procreations":800,"harness_ahead_so_far":1,"selection_tasks_completed_by_any":1,"#
    ));
}

#[test]
fn the_object_of_a_directed_evolution_that_has_ended_names_the_best_harness() {
    assert!(ended(&with_a_child_ahead()).json().contains(
        r#""max_procreations":800,"best_harness":1,"selection_tasks_completed_by_any":1,"#
    ));
}

#[test]
fn the_object_of_a_directed_evolution_that_has_not_ended_never_says_best_harness() {
    assert!(
        !so_far(&with_a_child_ahead())
            .json()
            .contains("best_harness")
    );
}

#[test]
fn the_object_of_a_directed_evolution_that_has_ended_names_no_harness_ahead_so_far() {
    assert!(!ended(&with_a_child_ahead()).json().contains("ahead_so_far"));
}

#[test]
fn the_object_holds_the_tasks_that_only_the_harness_that_is_ahead_so_far_completed() {
    let object = object_of(&so_far(&with_a_child_ahead()));
    assert_eq!(
        object["corroboration_lead"]["only_harness_ahead_so_far_completed"].as_u64(),
        Some(2)
    );
}

#[test]
fn the_object_holds_the_likelihood_of_the_lead_of_the_harness_that_is_ahead_so_far() {
    let object = object_of(&so_far(&with_a_child_ahead()));
    assert_eq!(
        object["corroboration_lead"]["likelihood_by_luck"].as_f64(),
        Some(0.25)
    );
}

#[test]
fn the_object_of_a_harness_ahead_so_far_that_is_the_primogenitor_holds_no_lead() {
    assert!(object_of(&so_far(&settings().genealogy()))["corroboration_lead"].is_null());
}

#[test]
fn the_lead_of_a_directed_evolution_that_has_not_ended_is_that_of_the_harness_ahead_so_far() {
    let lead = so_far(&with_a_child_ahead()).corroboration_lead.unwrap();
    assert_eq!(
        lead.only_leader_completed,
        OnlyLeaderCompleted::OnlyHarnessAheadSoFarCompleted(2)
    );
}
