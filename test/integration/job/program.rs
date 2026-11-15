use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::job;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn recorded() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let evaluator = test_inputs::sample("integration", "record.sh");
    let _ = job::run_evaluation(&workspace, &evaluator, 0, "t1", 30, &AtomicBool::new(false));
    (directory, workspace.evaluation(0, "t1"))
}

fn kept(file: &str) -> String {
    let (_directory, evaluation) = recorded();
    fs::read_to_string(evaluation.join(file)).unwrap()
}

#[test]
fn the_working_directory_of_a_program_is_its_output_directory() {
    let (_directory, evaluation) = recorded();
    assert!(evaluation.join("output/arguments").is_file());
}

#[test]
fn a_program_gets_the_environment_of_cladenc() {
    let path = format!("PATH={}", env::var("PATH").unwrap());
    assert!(kept("output/environment").lines().any(|line| line == path));
}

#[test]
fn the_standard_input_of_a_program_is_empty() {
    assert_eq!(kept("output/input"), "");
}

#[test]
fn the_standard_output_of_a_program_lands_in_stdout() {
    assert_eq!(kept("stdout"), "out\n");
}

#[test]
fn the_standard_error_of_a_program_lands_in_stderr() {
    assert_eq!(kept("stderr"), "err\n");
}
