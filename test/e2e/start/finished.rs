use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn options(directory: &Path) -> Vec<String> {
    [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--tasks", test_inputs::task_file(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        (
            "--evaluator",
            test_inputs::sample("e2e", "evaluate-listed.sh"),
        ),
        ("--run-dir", directory.join("run")),
        ("--budget", "10".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .iter()
    .map(|(option, value)| format!("{option}={}", value.display()))
    .collect()
}

fn ran(quiet: Option<&str>) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options(directory.path()))
        .args(quiet)
        .output()
        .unwrap();
    (directory, output)
}

#[test]
fn a_run_over_the_sample_programs_prints_the_directory_of_a_harness_on_standard_output() {
    let (_directory, output) = ran(None);
    assert!(Path::new(String::from_utf8(output.stdout).unwrap().trim_end()).is_dir());
}

#[test]
fn a_run_over_the_sample_programs_exits_with_0() {
    let (_directory, output) = ran(None);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_harness_a_run_prints_is_the_one_the_samples_make_best() {
    let (directory, output) = ran(None);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", directory.path().join("run/harnesses/1").display())
    );
}

#[test]
fn quiet_leaves_standard_error_empty() {
    let (_directory, output) = ran(Some("--quiet"));
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn without_quiet_standard_error_holds_the_records_of_the_journal() {
    let (directory, output) = ran(None);
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap()
    );
}
