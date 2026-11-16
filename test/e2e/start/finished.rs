use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn options(directory: &Path) -> Vec<String> {
    [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--run-dir", directory.join("run")),
        ("--budget", "10".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--assessment-timeout", "30".into()),
    ]
    .iter()
    .map(|(option, value)| format!("{option}={}", value.display()))
    .collect()
}

fn ran() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options(directory.path()))
        .output()
        .unwrap();
    (directory, output)
}

fn seed_in_the_run_record_of(directory: &TempDir) -> Option<u64> {
    let journal = fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap();
    let run_record: Value = serde_json::from_str(journal.lines().next().unwrap()).unwrap();
    run_record["seed"].as_u64()
}

#[test]
fn a_run_over_the_sample_programs_prints_the_directory_of_a_harness_on_standard_output() {
    let (_directory, output) = ran();
    assert!(Path::new(String::from_utf8(output.stdout).unwrap().trim_end()).is_dir());
}

#[test]
fn a_run_over_the_sample_programs_exits_with_0() {
    let (_directory, output) = ran();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_harness_a_run_prints_is_the_one_the_samples_make_best() {
    let (directory, output) = ran();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", directory.path().join("run/harnesses/1").display())
    );
}

#[test]
fn the_run_record_of_the_journal_holds_the_seed_that_cladenc_drew() {
    let (directory, _output) = ran();
    assert!(seed_in_the_run_record_of(&directory).is_some());
}

#[test]
fn two_runs_started_from_one_command_line_draw_different_seeds() {
    let (first, _output) = ran();
    let (second, _output) = ran();
    assert_ne!(
        seed_in_the_run_record_of(&first).unwrap(),
        seed_in_the_run_record_of(&second).unwrap()
    );
}

#[test]
fn every_line_of_standard_error_is_a_sentence_of_cladenc() {
    let (_directory, output) = ran();
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .all(|line| line.starts_with("cladenc: ") && !line.contains("\"record\""))
    );
}

#[test]
fn standard_error_starts_with_the_run_directory_being_made() {
    let (directory, output) = ran();
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with(&format!(
                "cladenc: making the run directory {} with a copy of the progenitor ",
                directory.path().join("run").display()
            ))
    );
}

#[test]
fn standard_error_ends_with_the_best_harness_and_where_it_is_kept() {
    let (directory, output) = ran();
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .ends_with(&format!(
                "its best harness is harness 1, kept in {}\n",
                directory.path().join("run/harnesses/1").display()
            ))
    );
}
