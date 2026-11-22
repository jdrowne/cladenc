use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

const CREATE_ONLY_WHERE_NO_CHILD_IS_GIVEN_TO_READ: &str = "if [ -d /var/log/cladenc/attempts ]; \
                                                           then [ ! -d /var/log/cladenc/children ] \
                                                           && echo procreated >> procreated; fi";

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
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            CREATE_ONLY_WHERE_NO_CHILD_IS_GIVEN_TO_READ.into(),
        ),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "3".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
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

fn report(directory: &Path, json: Option<&str>) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .args(json)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn experiments_of(directory: &TempDir) -> Value {
    let object: Value = serde_json::from_str(&report(directory.path(), Some("--json"))).unwrap();
    object["experiments"].clone()
}

#[test]
fn start_says_that_it_has_the_primogenitor_create_a_singleton_control_group_harness() {
    let (_directory, output) = ran();
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .any(|line| {
                line.starts_with("cladenc: started the creation of harness ")
            && line.contains(
                ", the singleton control group harness, by the primogenitor from the source \
                 code of harness ",
            )
            })
    );
}

#[test]
fn start_establishes_no_baseline_where_the_primogenitor_procreates() {
    let (directory, _output) = ran();
    assert!(
        !journal_of(&directory)
            .lines()
            .filter(|line| line.contains(r#""record":"baseline_started""#))
            .any(|line| line.ends_with(r#""progenitor":0}"#))
    );
}

#[test]
fn start_starts_as_many_procreations_as_the_most_it_was_given_beside_its_baselines() {
    let (directory, _output) = ran();
    assert_eq!(
        journal_of(&directory)
            .matches(r#""record":"procreation_started""#)
            .count(),
        3
    );
}

#[test]
fn start_benchmarks_a_singleton_control_group_harness_in_full() {
    let (directory, _output) = ran();
    let journal = journal_of(&directory);
    let singleton_control_group_harness: Value = journal
        .lines()
        .find(|line| line.contains(r#""record":"baseline_started""#))
        .map(|line| serde_json::from_str::<Value>(line).unwrap()["harness"].clone())
        .unwrap();
    assert_eq!(
        journal
            .matches(&format!(
                r#""record":"attempt_finished","harness":{singleton_control_group_harness},"#
            ))
            .count(),
        3
    );
}

#[test]
fn report_says_in_how_many_experiments_both_harnesses_were_created() {
    let (directory, _output) = ran();
    assert!(experiments_of(&directory)["both_created"].as_u64() >= Some(1));
}

#[test]
fn report_prints_what_the_experiments_created_above_the_table() {
    let (directory, _output) = ran();
    assert!(
        report(directory.path(), None)
            .lines()
            .nth(4)
            .unwrap()
            .starts_with("experiments: conducted ")
    );
}

#[test]
fn report_prints_what_told_the_harnesses_of_the_experiments_apart_above_the_table() {
    let (directory, _output) = ran();
    assert_eq!(
        report(directory.path(), None).lines().nth(5).unwrap(),
        "on the corroboration tasks of the experiments where both were created: only the \
         progenitor's child completed 0, only the singleton control group harness completed 0, \
         and luck alone gives a lead at least as large with likelihood 1.000"
    );
}
