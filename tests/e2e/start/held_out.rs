use std::fs;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn ran() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let options = [
        (
            "--primogenitor",
            test_inputs::primogenitor(directory.path()),
        ),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3", "t4"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--attempt-timeout", "30".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, output)
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap()
}

fn run_record_of(directory: &TempDir) -> Value {
    serde_json::from_str(journal_of(directory).lines().next().unwrap()).unwrap()
}

fn tasks_named_under(directory: &TempDir, key: &str) -> Vec<String> {
    run_record_of(directory)[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task.as_str().unwrap().to_owned())
        .collect()
}

fn holdout_tasks_of(directory: &TempDir) -> Vec<String> {
    tasks_named_under(directory, "holdout_tasks")
}

fn corroboration_tasks_of(directory: &TempDir) -> Vec<String> {
    tasks_named_under(directory, "corroboration_tasks")
}

fn the_one_task_the_child_completes_is_held_out(directory: &TempDir) -> bool {
    holdout_tasks_of(directory).contains(&"benchmark/t1".to_owned())
}

fn the_one_task_the_child_completes_is_a_corroboration_task(directory: &TempDir) -> bool {
    corroboration_tasks_of(directory).contains(&"benchmark/t1".to_owned())
}

fn the_one_task_the_child_completes_is_a_selection_task(directory: &TempDir) -> bool {
    the_one_task_the_child_completes_is_held_out(directory)
        && !the_one_task_the_child_completes_is_a_corroboration_task(directory)
}

fn report_of(directory: &TempDir) -> Value {
    let report = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .args(["report", "--json", "--outputs-dir"])
        .arg(directory.path().join("outputs"))
        .output()
        .unwrap();
    serde_json::from_slice(&report.stdout).unwrap()
}

fn tasks_kept_under(directory: &TempDir, kept: &str, harness: &str) -> Vec<String> {
    let benchmark = directory
        .path()
        .join("outputs")
        .join(kept)
        .join(harness)
        .join("benchmark");
    let mut tasks: Vec<String> = fs::read_dir(benchmark)
        .unwrap()
        .map(|entry| format!("benchmark/{}", entry.unwrap().file_name().to_string_lossy()))
        .collect();
    tasks.sort();
    tasks
}

fn tasks_not_held_out_of(directory: &TempDir) -> Vec<String> {
    let held_out = holdout_tasks_of(directory);
    ["t1", "t2", "t3", "t4"]
        .map(|id| format!("benchmark/{id}"))
        .into_iter()
        .filter(|task| !held_out.contains(task))
        .collect()
}

#[test]
fn a_run_that_holds_tasks_out_exits_with_0() {
    let (_directory, output) = ran();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_run_record_of_the_journal_holds_the_selection_tasks_percentage_that_was_given() {
    let (directory, _output) = ran();
    assert_eq!(
        run_record_of(&directory)["selection_tasks_percentage"].as_f64(),
        Some(25.0)
    );
}

#[test]
fn a_quarter_of_four_tasks_as_selection_and_a_quarter_as_corroboration_is_two_holdout_tasks() {
    let (directory, _output) = ran();
    assert_eq!(holdout_tasks_of(&directory).len(), 2);
}

#[test]
fn the_run_record_of_the_journal_holds_the_corroboration_tasks_percentage_that_was_given() {
    let (directory, _output) = ran();
    assert_eq!(
        run_record_of(&directory)["corroboration_tasks_percentage"].as_f64(),
        Some(25.0)
    );
}

#[test]
fn a_quarter_of_a_benchmark_of_four_tasks_is_one_corroboration_task() {
    let (directory, _output) = ran();
    assert_eq!(corroboration_tasks_of(&directory).len(), 1);
}

#[test]
fn the_corroboration_task_is_one_of_the_holdout_tasks() {
    let (directory, _output) = ran();
    assert!(holdout_tasks_of(&directory).contains(&corroboration_tasks_of(&directory)[0]));
}

#[test]
fn a_run_says_how_many_tasks_of_its_benchmark_it_holds_out() {
    let (directory, output) = ran();
    let given_in = fs::canonicalize(directory.path()).unwrap();
    assert!(String::from_utf8(output.stderr).unwrap().contains(&format!(
        "cladenc: holding out 2 of the 4 tasks of the benchmark {} from every progenitor: 1 \
         selection task and 1 corroboration task\n",
        given_in.join("benchmark").display()
    )));
}

#[test]
fn the_attempts_of_the_primogenitor_at_the_holdout_tasks_are_kept_under_holdout() {
    let (directory, _output) = ran();
    assert_eq!(
        tasks_kept_under(&directory, "holdout", "0"),
        holdout_tasks_of(&directory)
    );
}

#[test]
fn the_attempts_of_a_child_at_the_holdout_tasks_are_kept_under_holdout() {
    let (directory, _output) = ran();
    assert_eq!(
        tasks_kept_under(&directory, "holdout", "1"),
        holdout_tasks_of(&directory)
    );
}

#[test]
fn the_directory_a_progenitor_is_given_holds_its_attempts_at_the_tasks_that_are_not_held_out() {
    let (directory, _output) = ran();
    assert_eq!(
        tasks_kept_under(&directory, "attempts", "0"),
        tasks_not_held_out_of(&directory)
    );
}

#[test]
fn every_harness_is_benchmarked_on_the_holdout_tasks_and_on_the_others() {
    let (directory, _output) = ran();
    assert_eq!(
        journal_of(&directory)
            .matches(r#""record":"attempt_finished""#)
            .count(),
        8
    );
}

#[test]
fn the_child_is_named_the_best_harness_only_where_the_one_task_it_completes_is_a_selection_task() {
    let (directory, _output) = ran();
    assert_eq!(
        report_of(&directory)["best_harness"].as_u64(),
        Some(u64::from(
            the_one_task_the_child_completes_is_a_selection_task(&directory)
        ))
    );
}

#[test]
fn the_report_counts_a_task_a_harness_completed_as_a_selection_task_only_where_it_is_one() {
    let (directory, _output) = ran();
    assert_eq!(
        report_of(&directory)["harnesses"][1]["selection_completed"].as_u64(),
        Some(u64::from(
            the_one_task_the_child_completes_is_a_selection_task(&directory)
        ))
    );
}

#[test]
fn the_report_counts_a_task_a_harness_completed_as_a_corroboration_task_only_where_it_is_one() {
    let (directory, _output) = ran();
    assert_eq!(
        report_of(&directory)["harnesses"][1]["corroboration_completed"].as_u64(),
        Some(u64::from(
            the_one_task_the_child_completes_is_a_corroboration_task(&directory)
        ))
    );
}

#[test]
fn the_report_counts_a_task_a_harness_completed_as_a_non_holdout_task_where_it_is_not_held_out() {
    let (directory, _output) = ran();
    assert_eq!(
        report_of(&directory)["harnesses"][1]["non_holdout_completed"].as_u64(),
        Some(u64::from(!the_one_task_the_child_completes_is_held_out(
            &directory
        )))
    );
}

#[test]
fn the_report_counts_one_selection_task_one_corroboration_task_and_two_non_holdout_tasks_for_each_harness()
 {
    let (directory, _output) = ran();
    let report = report_of(&directory);
    assert_eq!(
        [0, 1].map(|harness: usize| {
            ["selection", "corroboration", "non_holdout"].map(|set| {
                ["completed", "not_completed"]
                    .map(|result| {
                        report["harnesses"][harness][format!("{set}_{result}")]
                            .as_u64()
                            .unwrap()
                    })
                    .iter()
                    .sum::<u64>()
            })
        }),
        [[1, 1, 2], [1, 1, 2]]
    );
}

#[test]
fn the_report_counts_the_tasks_of_each_set_that_at_least_one_harness_completed() {
    let (directory, _output) = ran();
    let report = report_of(&directory);
    assert_eq!(
        [
            "selection_tasks",
            "corroboration_tasks",
            "non_holdout_tasks"
        ]
        .map(|set| report[set].as_u64().unwrap()),
        [1, 1, 2]
    );
}
