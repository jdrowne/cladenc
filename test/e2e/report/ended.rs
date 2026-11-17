use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use cladenc::artificial_selection::lower_bound;
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
        ("--run-dir", directory.path().join("run")),
        ("--budget", "10".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
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
        .arg(format!("--run-dir={}", directory.join("run").display()))
        .args(json)
        .output()
        .unwrap()
}

fn number(value: f64) -> String {
    serde_json::to_string(&value).unwrap()
}

#[test]
fn report_prints_the_table() {
    let directory = ended();
    assert_eq!(
        String::from_utf8(report(directory.path(), None).stdout).unwrap(),
        "model a-model at http://models.example/v1\n\
         started 2 of 10, best harness 1\n\
         \n\
         id  progenitor  state    completed  not completed  clade completed  clade not completed  clade score  lower bound\n\
         0   -           created  0          1              1                1                    0.500        0.005\n\
         1   0           created  1          0              1                0                    1.000        0.100\n"
    );
}

#[test]
fn with_json_report_prints_an_object_holding_the_same_values() {
    let directory = ended();
    assert_eq!(
        String::from_utf8(report(directory.path(), Some("--json")).stdout).unwrap(),
        format!(
            concat!(
                r#"{{"model":"a-model","model_url":"http://models.example/v1","#,
                r#""started":2,"budget":10,"best_harness":1,"harnesses":["#,
                r#"{{"id":0,"progenitor":null,"state":"created","completed":0,"not_completed":1,"#,
                r#""clade_completed":1,"clade_not_completed":1,"clade_score":0.5,"#,
                r#""lower_bound":{}}},"#,
                r#"{{"id":1,"progenitor":0,"state":"created","completed":1,"not_completed":0,"#,
                r#""clade_completed":1,"clade_not_completed":0,"clade_score":1.0,"#,
                r#""lower_bound":{}}}]}}"#,
                "\n"
            ),
            number(lower_bound(0, 1, 99.0)),
            number(lower_bound(1, 0, 99.0))
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
    let journal = directory.path().join("run/journal.jsonl");
    let before = fs::read(&journal).unwrap();
    report(directory.path(), None);
    assert_eq!(fs::read(&journal).unwrap(), before);
}
