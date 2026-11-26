use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{CreationOutcome, Experiment, Genealogy, HarnessId, TrialResult};
use crate::journal::Settings;
use crate::report::Report;
use serde_json::Value;

const CREATED: CreationOutcome = CreationOutcome::Created;
const FAILED: CreationOutcome = CreationOutcome::Failed;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: ["c1", "c2", "c3", "o1", "s1"].map(str::to_owned).to_vec(),
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
        selection_tasks_percentage: 20.0,
        holdout_tasks: ["c1", "c2", "c3", "s1"]
            .map(|id| format!("benchmark/{id}"))
            .to_vec(),
        corroboration_tasks_percentage: 60.0,
        corroboration_tasks: ["c1", "c2", "c3"]
            .map(|id| format!("benchmark/{id}"))
            .to_vec(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn benchmarked(genealogy: &mut Genealogy, harness: HarnessId, completed: &[&str]) {
    for task in settings().tasks() {
        let result = if completed.contains(&task.as_str()) {
            TrialResult::Pass
        } else {
            TrialResult::Fail
        };
        genealogy.start_trial(harness, &task).unwrap();
        genealogy.end_trial(harness, &task, result).unwrap();
    }
}

fn experiment_on(
    genealogy: &mut Genealogy,
    progenitor: HarnessId,
    outcomes: (CreationOutcome, CreationOutcome),
) -> (HarnessId, HarnessId) {
    let child = genealogy.start_diversification(progenitor).unwrap();
    genealogy.end_diversification(child, outcomes.0).unwrap();
    let singleton_control_group_harness = genealogy
        .start_baseline(Experiment { child, progenitor })
        .unwrap();
    genealogy
        .end_baseline(singleton_control_group_harness, outcomes.1)
        .unwrap();
    (child, singleton_control_group_harness)
}

fn with_a_child_of_the_primogenitor() -> Genealogy {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy.end_diversification(child, CREATED).unwrap();
    genealogy
}

fn after_an_experiment(child: &[&str], singleton_control_group_harness: &[&str]) -> Genealogy {
    let mut genealogy = with_a_child_of_the_primogenitor();
    experiment_on(&mut genealogy, 1, (CREATED, CREATED));
    benchmarked(&mut genealogy, 2, child);
    benchmarked(&mut genealogy, 3, singleton_control_group_harness);
    genealogy
}

fn after_two_experiments() -> Genealogy {
    let mut genealogy = after_an_experiment(&["benchmark/c1", "benchmark/c2"], &["benchmark/c2"]);
    experiment_on(&mut genealogy, 2, (CREATED, CREATED));
    benchmarked(&mut genealogy, 4, &["benchmark/c1"]);
    benchmarked(&mut genealogy, 5, &["benchmark/c2"]);
    genealogy
}

fn after_an_experiment_that_created(outcomes: (CreationOutcome, CreationOutcome)) -> Genealogy {
    let mut genealogy = with_a_child_of_the_primogenitor();
    let (child, singleton_control_group_harness) = experiment_on(&mut genealogy, 1, outcomes);
    for (harness, outcome) in [
        (child, outcomes.0),
        (singleton_control_group_harness, outcomes.1),
    ] {
        if outcome == CREATED {
            benchmarked(&mut genealogy, harness, &["benchmark/c1"]);
        }
    }
    genealogy
}

fn report_of(genealogy: &Genealogy) -> Report {
    Report::of(&settings(), genealogy, &[], Some(0))
}

fn told_apart(genealogy: &Genealogy) -> (u64, u64) {
    let experiments = report_of(genealogy).experiments;
    (
        experiments.only_progenitors_child_completed,
        experiments.only_singleton_control_group_harness_completed,
    )
}

fn created(genealogy: &Genealogy) -> (u64, u64, u64) {
    let experiments = report_of(genealogy).experiments;
    (
        experiments.progenitors_child_created,
        experiments.singleton_control_group_harness_created,
        experiments.both_created,
    )
}

fn object_of(genealogy: &Genealogy) -> Value {
    serde_json::from_str(&report_of(genealogy).json()).unwrap()
}

fn line_of(genealogy: &Genealogy, line: usize) -> String {
    report_of(genealogy)
        .table()
        .lines()
        .nth(line)
        .unwrap()
        .to_owned()
}

#[test]
fn a_directed_evolution_in_which_only_the_primogenitor_was_cloned_conducted_no_experiment() {
    let report = report_of(&with_a_child_of_the_primogenitor());
    assert_eq!(report.experiments.conducted, 0);
}

#[test]
fn each_baseline_that_was_started_is_an_experiment_that_was_conducted() {
    assert_eq!(report_of(&after_two_experiments()).experiments.conducted, 2);
}

#[test]
fn the_corroboration_tasks_that_only_one_of_the_two_harnesses_of_an_experiment_completed_are_counted()
 {
    assert_eq!(
        told_apart(&after_an_experiment(
            &["benchmark/c1", "benchmark/c2"],
            &["benchmark/c2", "benchmark/c3"]
        )),
        (1, 1)
    );
}

#[test]
fn a_task_that_is_no_corroboration_task_tells_the_two_harnesses_of_an_experiment_apart_by_nothing()
{
    assert_eq!(
        told_apart(&after_an_experiment(&["benchmark/s1", "benchmark/o1"], &[])),
        (0, 0)
    );
}

#[test]
fn the_counts_of_every_experiment_are_added_up() {
    assert_eq!(told_apart(&after_two_experiments()), (2, 1));
}

#[test]
fn an_experiment_whose_two_harnesses_were_created_counts_for_each_and_for_both() {
    assert_eq!(
        created(&after_an_experiment_that_created((CREATED, CREATED))),
        (1, 1, 1)
    );
}

#[test]
fn an_experiment_whose_progenitor_s_child_was_not_created_counts_for_the_other_harness_alone() {
    assert_eq!(
        created(&after_an_experiment_that_created((FAILED, CREATED))),
        (0, 1, 0)
    );
}

#[test]
fn an_experiment_whose_singleton_control_group_harness_was_not_created_counts_for_the_child_alone()
{
    assert_eq!(
        created(&after_an_experiment_that_created((CREATED, FAILED))),
        (1, 0, 0)
    );
}

#[test]
fn the_tasks_of_an_experiment_where_one_of_the_two_harnesses_was_not_created_are_not_counted() {
    assert_eq!(
        told_apart(&after_an_experiment_that_created((FAILED, CREATED))),
        (0, 0)
    );
}

#[test]
fn a_task_that_one_of_the_two_harnesses_has_not_trialled_yet_is_not_counted() {
    let mut genealogy = with_a_child_of_the_primogenitor();
    experiment_on(&mut genealogy, 1, (CREATED, CREATED));
    benchmarked(&mut genealogy, 2, &["benchmark/c1"]);
    assert_eq!(told_apart(&genealogy), (0, 0));
}

#[test]
fn the_likelihood_is_that_of_a_lead_of_the_progenitors_children_at_least_as_large_by_luck() {
    let report = report_of(&after_an_experiment(
        &["benchmark/c1", "benchmark/c2", "benchmark/c3"],
        &[],
    ));
    assert_eq!(report.experiments.likelihood_by_luck, Some(0.125));
}

#[test]
fn where_no_task_tells_the_harnesses_of_the_experiments_apart_the_likelihood_is_1() {
    let report = report_of(&after_an_experiment(&[], &[]));
    assert_eq!(report.experiments.likelihood_by_luck, Some(1.0));
}

#[test]
fn where_no_experiment_was_conducted_there_is_no_likelihood() {
    let report = report_of(&with_a_child_of_the_primogenitor());
    assert_eq!(report.experiments.likelihood_by_luck, None);
}

#[test]
fn where_no_experiment_created_both_harnesses_there_is_no_likelihood() {
    let report = report_of(&after_an_experiment_that_created((CREATED, FAILED)));
    assert_eq!(report.experiments.likelihood_by_luck, None);
}

#[test]
fn a_baseline_is_not_reported_as_a_diversification_that_was_started() {
    assert_eq!(
        report_of(&after_two_experiments()).diversifications_started,
        3
    );
}

#[test]
fn a_task_that_only_a_singleton_control_group_harness_completed_is_completed_by_no_harness() {
    let report = report_of(&after_an_experiment(&[], &["benchmark/s1"]));
    assert_eq!(report.selection_tasks_completed_by_any, 0);
}

#[test]
fn the_table_says_how_many_experiments_were_conducted_and_what_they_created() {
    assert_eq!(
        line_of(&after_two_experiments(), 4),
        "experiments: conducted 2, the progenitor's child created in 2, the singleton control \
         group harness created in 2, both created in 2"
    );
}

#[test]
fn the_table_says_what_told_the_harnesses_of_the_experiments_apart_and_how_likely_by_luck() {
    assert_eq!(
        line_of(&after_two_experiments(), 5),
        "on the corroboration tasks of the experiments where both were created: only the \
         progenitor's child completed 2, only the singleton control group harness completed 1, \
         and luck alone gives a lead at least as large with likelihood 0.500"
    );
}

#[test]
fn the_table_says_that_no_experiment_created_both_harnesses_where_none_did() {
    assert_eq!(
        line_of(&after_an_experiment_that_created((FAILED, CREATED)), 5),
        "on the corroboration tasks of the experiments: no experiment created both the \
         progenitor's child and the singleton control group harness, so there are no two \
         harnesses to compare"
    );
}

#[test]
fn the_table_still_says_what_the_experiments_created_where_none_created_both_harnesses() {
    assert_eq!(
        line_of(&after_an_experiment_that_created((FAILED, CREATED)), 4),
        "experiments: conducted 1, the progenitor's child created in 0, the singleton control \
         group harness created in 1, both created in 0"
    );
}

#[test]
fn the_row_of_a_singleton_control_group_harness_names_no_progenitor_and_the_child_of_its_experiment()
 {
    assert!(
        line_of(&after_two_experiments(), 11).starts_with("3   -           2           created  ")
    );
}

#[test]
fn the_row_of_a_child_names_no_experiment() {
    assert!(
        line_of(&after_two_experiments(), 10).starts_with("2   1           -           created  ")
    );
}

#[test]
fn the_object_holds_the_counts_of_the_experiments() {
    assert_eq!(
        object_of(&after_two_experiments())["experiments"],
        serde_json::json!({
            "conducted": 2,
            "progenitors_child_created": 2,
            "singleton_control_group_harness_created": 2,
            "both_created": 2,
            "only_progenitors_child_completed": 2,
            "only_singleton_control_group_harness_completed": 1,
            "likelihood_by_luck": 0.5
        })
    );
}

#[test]
fn the_object_names_the_experiment_of_a_singleton_control_group_harness() {
    assert_eq!(
        object_of(&after_two_experiments())["harnesses"][5]["experiment"],
        serde_json::json!({"child": 4, "progenitor": 2})
    );
}

#[test]
fn the_object_names_no_experiment_for_a_child() {
    assert!(object_of(&after_two_experiments())["harnesses"][4]["experiment"].is_null());
}

#[test]
fn the_object_gives_no_likelihood_where_no_experiment_created_both_harnesses() {
    let object = object_of(&after_an_experiment_that_created((CREATED, FAILED)));
    assert!(object["experiments"]["likelihood_by_luck"].is_null());
}

#[test]
fn the_object_still_holds_the_counts_where_no_experiment_created_both_harnesses() {
    assert_eq!(
        object_of(&after_an_experiment_that_created((CREATED, FAILED)))["experiments"],
        serde_json::json!({
            "conducted": 1,
            "progenitors_child_created": 1,
            "singleton_control_group_harness_created": 0,
            "both_created": 0,
            "only_progenitors_child_completed": 0,
            "only_singleton_control_group_harness_completed": 0,
            "likelihood_by_luck": null
        })
    );
}
