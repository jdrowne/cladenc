use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

use crate::shared::ENDED;

fn report(directory: &Path, json: Option<&str>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .args(json)
        .output()
        .unwrap()
}

fn table_of(directory: &Path) -> String {
    String::from_utf8(report(directory, None).stdout).unwrap()
}

fn object_of(directory: &Path) -> Value {
    serde_json::from_slice(&report(directory, Some("--json")).stdout).unwrap()
}

fn up_to_the_seconds(table: &str) -> String {
    let seconds = table
        .lines()
        .nth(7)
        .and_then(|header| header.find("attempt seconds"))
        .unwrap();
    let above: String = table
        .lines()
        .take(7)
        .map(|line| format!("{line}\n"))
        .collect();
    let rows: String = table
        .lines()
        .skip(7)
        .map(|line| format!("{}\n", line.get(..seconds).unwrap_or(line).trim_end()))
        .collect();
    above + &rows
}

fn without_the_seconds(mut object: Value) -> Value {
    for harness in object["harnesses"].as_array_mut().unwrap() {
        let values = harness.as_object_mut().unwrap();
        values.remove("attempt_seconds");
        values.remove("procreation_seconds");
    }
    object
}

#[test]
fn report_prints_the_table() {
    let (directory, _run) = &*ENDED;
    assert_eq!(
        up_to_the_seconds(&table_of(directory.path())),
        "model a-model at http://models.example/v1\n\
         started 1 of 1 procreation, best harness 1\n\
         completed by at least one harness: selection tasks 1 of 1, corroboration tasks 1 of 1, non-holdout tasks 1 of 1\n\
         on the corroboration tasks: only the best harness completed 1, only the primogenitor completed 0, and luck alone gives a lead at least as large with likelihood 0.500\n\
         experiments: conducted 0, the progenitor's child created in 0, the singleton control group harness created in 0, both created in 0\n\
         on the corroboration tasks of the experiments: no experiment created both the progenitor's child and the singleton control group harness, so there are no two harnesses to compare\n\
         \n\
         id  progenitor  experiment  state    selection completed  selection not completed  corroboration completed  corroboration not completed  non-holdout completed  non-holdout not completed  clade completed  clade not completed  clade score\n\
         0   -           -           created  0                    1                        0                        1                            0                      1                          1                1                    0.500\n\
         1   0           -           created  1                    0                        1                        0                            1                      0                          1                0                    1.000\n"
    );
}

#[test]
fn with_json_report_prints_an_object_holding_the_same_values() {
    let (directory, _run) = &*ENDED;
    let values: Value = serde_json::from_str(concat!(
        r#"{"ended":true,"model":"a-model","model_url":"http://models.example/v1","#,
        r#""procreations_started":1,"max_procreations":1,"best_harness":1,"#,
        r#""selection_tasks_completed_by_any":1,"selection_tasks":1,"#,
        r#""corroboration_tasks_completed_by_any":1,"corroboration_tasks":1,"#,
        r#""non_holdout_tasks_completed_by_any":1,"non_holdout_tasks":1,"#,
        r#""corroboration_lead":{"only_best_harness_completed":1,"#,
        r#""only_primogenitor_completed":0,"likelihood_by_luck":0.5},"#,
        r#""experiments":{"conducted":0,"progenitors_child_created":0,"#,
        r#""singleton_control_group_harness_created":0,"both_created":0,"#,
        r#""only_progenitors_child_completed":0,"#,
        r#""only_singleton_control_group_harness_completed":0,"likelihood_by_luck":null},"#,
        r#""harnesses":["#,
        r#"{"id":0,"progenitor":null,"experiment":null,"state":"created","#,
        r#""selection_completed":0,"selection_not_completed":1,"#,
        r#""corroboration_completed":0,"corroboration_not_completed":1,"#,
        r#""non_holdout_completed":0,"non_holdout_not_completed":1,"#,
        r#""clade_completed":1,"clade_not_completed":1,"clade_score":0.5},"#,
        r#"{"id":1,"progenitor":0,"experiment":null,"state":"created","#,
        r#""selection_completed":1,"selection_not_completed":0,"#,
        r#""corroboration_completed":1,"corroboration_not_completed":0,"#,
        r#""non_holdout_completed":1,"non_holdout_not_completed":0,"#,
        r#""clade_completed":1,"clade_not_completed":0,"clade_score":1.0}]}"#,
    ))
    .unwrap();
    assert_eq!(without_the_seconds(object_of(directory.path())), values);
}

#[test]
fn the_table_of_a_report_ends_with_the_seconds_of_the_attempts_and_of_the_procreation() {
    let (directory, _run) = &*ENDED;
    assert!(
        table_of(directory.path())
            .lines()
            .nth(7)
            .unwrap()
            .ends_with("  attempt seconds  procreation seconds")
    );
}

#[test]
fn report_shows_the_seconds_that_the_attempts_of_each_harness_took_together() {
    let (directory, _run) = &*ENDED;
    let object = object_of(directory.path());
    assert_eq!(
        [0, 1].map(|harness: usize| {
            object["harnesses"][harness]["attempt_seconds"]
                .as_f64()
                .unwrap()
                > 0.0
        }),
        [true, true]
    );
}

#[test]
fn report_shows_the_seconds_that_the_procreation_that_made_a_child_took() {
    let (directory, _run) = &*ENDED;
    assert!(
        object_of(directory.path())["harnesses"][1]["procreation_seconds"]
            .as_f64()
            .unwrap()
            > 0.0
    );
}

#[test]
fn report_shows_no_seconds_of_a_procreation_for_the_primogenitor() {
    let (directory, _run) = &*ENDED;
    assert!(object_of(directory.path())["harnesses"][0]["procreation_seconds"].is_null());
}

#[test]
fn with_json_report_prints_the_object_on_one_line() {
    let (directory, _run) = &*ENDED;
    assert_eq!(
        String::from_utf8(report(directory.path(), Some("--json")).stdout)
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn report_exits_with_0() {
    let (directory, _run) = &*ENDED;
    assert_eq!(report(directory.path(), None).status.code(), Some(0));
}

#[test]
fn report_leaves_the_journal_unchanged() {
    let (directory, _run) = &*ENDED;
    let journal = directory.path().join("outputs/journal.jsonl");
    let before = fs::read(&journal).unwrap();
    report(directory.path(), None);
    assert_eq!(fs::read(&journal).unwrap(), before);
}
