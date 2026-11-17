use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn ended() -> TempDir {
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
            test_inputs::benchmark(directory.path(), &["t1"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--genealogy-growth", "0".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--attempt-timeout", "30".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    directory
}

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

#[test]
fn report_prints_the_table() {
    let directory = ended();
    assert_eq!(
        String::from_utf8(report(directory.path(), None).stdout).unwrap(),
        "model a-model at http://models.example/v1\n\
         started 1 of 1 procreation, best harness 1\n\
         \n\
         id  progenitor  state    completed  not completed  clade completed  clade not completed  clade score\n\
         0   -           created  0          1              1                1                    0.500\n\
         1   0           created  1          0              1                0                    1.000\n"
    );
}

#[test]
fn with_json_report_prints_an_object_holding_the_same_values() {
    let directory = ended();
    assert_eq!(
        String::from_utf8(report(directory.path(), Some("--json")).stdout).unwrap(),
        concat!(
            r#"{"model":"a-model","model_url":"http://models.example/v1","#,
            r#""procreations_started":1,"max_procreations":1,"best_harness":1,"harnesses":["#,
            r#"{"id":0,"progenitor":null,"state":"created","completed":0,"not_completed":1,"#,
            r#""clade_completed":1,"clade_not_completed":1,"clade_score":0.5},"#,
            r#"{"id":1,"progenitor":0,"state":"created","completed":1,"not_completed":0,"#,
            r#""clade_completed":1,"clade_not_completed":0,"clade_score":1.0}]}"#,
            "\n"
        )
    );
}

#[test]
fn report_exits_with_0() {
    let directory = ended();
    assert_eq!(report(directory.path(), None).status.code(), Some(0));
}

#[test]
fn report_leaves_the_journal_unchanged() {
    let directory = ended();
    let journal = directory.path().join("outputs/journal.jsonl");
    let before = fs::read(&journal).unwrap();
    report(directory.path(), None);
    assert_eq!(fs::read(&journal).unwrap(), before);
}
