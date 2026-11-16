use std::fs;
use std::path::{Path, PathBuf};

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::Score;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, workspace)
}

fn started() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    workspace.start_assessment(0, "benchmark/t1").unwrap();
    (directory, workspace)
}

fn ended(score: Score) -> (TempDir, Workspace) {
    let (directory, workspace) = started();
    workspace.end_assessment(0, "benchmark/t1", score).unwrap();
    (directory, workspace)
}

fn score_written(score: Score) -> String {
    let (_directory, workspace) = ended(score);
    fs::read_to_string(workspace.assessment(0, "benchmark/t1").join("score")).unwrap()
}

fn held_while_running(file: &str) -> String {
    let (_directory, workspace) = started();
    fs::read_to_string(workspace.running_assessment(0, "benchmark/t1").join(file)).unwrap()
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn a_started_assessment_holds_a_copy_of_its_harness() {
    assert_eq!(
        held_while_running("harness/harness.sh"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_started_assessment_holds_a_copy_of_the_files_of_its_task_to_work_on() {
    assert_eq!(held_while_running("work/work.txt"), "to do\n");
}

#[test]
fn a_started_assessment_holds_the_statement_of_its_task() {
    assert_eq!(held_while_running("statement"), "Finish the work of t1.\n");
}

#[test]
fn a_started_assessment_holds_an_empty_directory_for_what_the_check_prints() {
    let (_directory, workspace) = started();
    assert_eq!(
        entries(
            workspace
                .running_assessment(0, "benchmark/t1")
                .join("check")
        ),
        0
    );
}

#[test]
fn a_started_assessment_holds_no_copy_of_the_check() {
    let (_directory, workspace) = started();
    assert!(
        !workspace
            .running_assessment(0, "benchmark/t1")
            .join("work/check")
            .exists()
    );
}

#[test]
fn a_started_assessment_has_an_empty_directory_for_its_benchmark_among_those_of_its_harness() {
    let (_directory, workspace) = started();
    assert_eq!(entries(workspace.assessments(0).join("benchmark")), 0);
}

#[test]
fn an_assessment_on_a_task_the_run_directory_does_not_hold_cannot_be_started() {
    let (_directory, workspace) = made();
    assert!(workspace.start_assessment(0, "benchmark/t9").is_err());
}

#[test]
fn a_harness_scored_a_success_leaves_the_word_success_as_its_score() {
    assert_eq!(score_written(Score::Success), "success\n");
}

#[test]
fn a_harness_scored_a_failure_leaves_the_word_failure_as_its_score() {
    assert_eq!(score_written(Score::Failure), "failure\n");
}

#[test]
fn a_harness_whose_assessment_timed_out_leaves_the_word_timed_out_as_its_score() {
    assert_eq!(score_written(Score::TimedOut), "timed_out\n");
}

#[test]
fn an_ended_assessment_keeps_no_copy_of_its_harness() {
    let (_directory, workspace) = ended(Score::Success);
    assert!(
        !workspace
            .assessment(0, "benchmark/t1")
            .join("harness")
            .exists()
    );
}

#[test]
fn an_ended_assessment_keeps_the_work() {
    let (_directory, workspace) = ended(Score::Success);
    assert!(
        workspace
            .assessment(0, "benchmark/t1")
            .join("work/work.txt")
            .is_file()
    );
}

#[test]
fn an_ended_assessment_keeps_the_statement_of_its_task() {
    let (_directory, workspace) = ended(Score::Success);
    assert!(
        workspace
            .assessment(0, "benchmark/t1")
            .join("statement")
            .is_file()
    );
}

#[test]
fn an_ended_assessment_leaves_nothing_running() {
    let (_directory, workspace) = ended(Score::Success);
    assert!(!workspace.running_assessment(0, "benchmark/t1").exists());
}

#[test]
fn the_directory_of_an_ended_assessment_is_named_after_its_harness_its_benchmark_and_its_task() {
    assert_eq!(
        Workspace::at(Path::new("/run")).assessment(3, "parsing/fix-the-parser"),
        Path::new("/run/assessments/3/parsing/fix-the-parser")
    );
}

#[test]
fn the_directory_of_a_running_assessment_names_its_benchmark_and_its_task_in_one_name() {
    assert_eq!(
        Workspace::at(Path::new("/run")).running_assessment(3, "parsing/fix-the-parser"),
        Path::new("/run/running/assessment-3-parsing+fix-the-parser")
    );
}

#[test]
fn an_assessment_that_ends_after_another_on_a_task_of_the_same_benchmark_is_kept_beside_it() {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    for task in ["benchmark/t1", "benchmark/t2"] {
        workspace.start_assessment(0, task).unwrap();
        workspace.end_assessment(0, task, Score::Success).unwrap();
    }
    assert_eq!(entries(workspace.assessments(0).join("benchmark")), 2);
}
