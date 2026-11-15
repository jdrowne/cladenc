use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use cladenc::policy::lower_bound;
use tempfile::TempDir;

fn ended() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let options = [
        ("--progenitor", test_inputs::progenitor(directory.path())),
        ("--tasks", test_inputs::task_file(directory.path(), &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        (
            "--evaluator",
            test_inputs::sample("e2e", "evaluate-listed.sh"),
        ),
        ("--run-dir", directory.path().join("run")),
        ("--budget", "10".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("run")
        .args(options)
        .arg("--quiet")
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
        "started 2 of 10, best harness 1\n\
         \n\
         id  parent  state    successes  failures  clade successes  clade failures  clade success rate  lower bound\n\
         0   -       created  0          1         1                1               0.500               0.005\n\
         1   0       created  1          0         1                0               1.000               0.100\n"
    );
}

#[test]
fn with_json_report_prints_an_object_holding_the_same_values() {
    let directory = ended();
    assert_eq!(
        String::from_utf8(report(directory.path(), Some("--json")).stdout).unwrap(),
        format!(
            concat!(
                r#"{{"started":2,"budget":10,"best_harness":1,"harnesses":["#,
                r#"{{"id":0,"parent":null,"state":"created","successes":0,"failures":1,"#,
                r#""clade_successes":1,"clade_failures":1,"clade_success_rate":0.5,"#,
                r#""lower_bound":{}}},"#,
                r#"{{"id":1,"parent":0,"state":"created","successes":1,"failures":0,"#,
                r#""clade_successes":1,"clade_failures":0,"clade_success_rate":1.0,"#,
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
