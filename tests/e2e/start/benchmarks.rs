use std::fs;
use std::path::Path;
use std::process::{Command, Output};

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
            test_inputs::benchmark_named(directory.path(), "refactoring", &["t1", "t2", "t3"]),
        ),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
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

fn result_of(directory: &TempDir, task: &str) -> String {
    let kept = test_inputs::kept_attempt(&directory.path().join("outputs"), 1, task);
    fs::read_to_string(kept.join("result")).unwrap()
}

fn tasks_attempted_of(directory: &TempDir, benchmark: &str) -> Vec<String> {
    let mut tasks: Vec<String> = ["attempts", "holdout"]
        .iter()
        .flat_map(|kept| {
            names_in(
                &directory
                    .path()
                    .join("outputs")
                    .join(kept)
                    .join("1")
                    .join(benchmark),
            )
        })
        .collect();
    tasks.sort();
    tasks
}

fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_run_given_two_benchmarks_exits_with_0() {
    let (_directory, output) = ran();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_run_given_two_benchmarks_keeps_the_tasks_of_each_under_its_name() {
    let (directory, _output) = ran();
    assert_eq!(
        ["benchmark", "refactoring"]
            .map(|name| { names_in(&directory.path().join("outputs/benchmarks").join(name)) }),
        [vec!["t1", "t2", "t3"], vec!["t1", "t2", "t3"]]
    );
}

#[test]
fn a_run_given_two_benchmarks_says_that_it_copies_the_tasks_of_each_in_the_order_of_their_names() {
    let (directory, output) = ran();
    let said = String::from_utf8(output.stderr).unwrap();
    let given_in = fs::canonicalize(directory.path()).unwrap();
    assert_eq!(
        said.lines().skip(1).take(2).collect::<Vec<_>>(),
        [
            format!(
                "cladenc: copying the 3 tasks of the benchmark {} into the outputs directory",
                given_in.join("benchmark").display()
            ),
            format!(
                "cladenc: copying the 3 tasks of the benchmark {} into the outputs directory",
                given_in.join("refactoring").display()
            )
        ]
    );
}

#[test]
fn a_run_given_two_benchmarks_tries_a_harness_on_the_tasks_of_both() {
    let (directory, _output) = ran();
    assert_eq!(
        ["benchmark", "refactoring"].map(|name| tasks_attempted_of(&directory, name)),
        [vec!["t1", "t2", "t3"], vec!["t1", "t2", "t3"]]
    );
}

#[test]
fn two_tasks_of_the_same_id_in_two_benchmarks_are_attempted_apart() {
    let (directory, _output) = ran();
    assert_eq!(
        (
            result_of(&directory, "benchmark/t1"),
            result_of(&directory, "refactoring/t1")
        ),
        ("completed\n".to_owned(), "completed\n".to_owned())
    );
}

#[test]
fn the_journal_of_a_run_given_two_benchmarks_names_a_task_by_its_benchmark_and_its_id() {
    let (directory, _output) = ran();
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
    assert!(
        journal.contains(r#"{"record":"attempt_started","harness":1,"task":"refactoring/t1"}"#)
    );
}
