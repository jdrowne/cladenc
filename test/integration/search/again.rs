use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::journal::{self, Journal, Record, Settings};
use cladenc::search;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn settings(directory: &Path, evaluator: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        tasks: vec!["t1".to_owned()],
        docker_image: test_inputs::IMAGE.to_owned(),
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        evaluator: test_inputs::sample("integration", evaluator),
        budget: 4,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 1,
        workers: 1,
        procreation_timeout: 60,
        evaluation_timeout: 60,
        seed: 7,
    }
}

fn resumed_after_a_stop_at(
    solved: u64,
    task: &str,
    leftover: &str,
    initial_procreations: u64,
) -> TempDir {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), "evaluate-listed-or-exit-2.sh");
    settings.initial_procreations = initial_procreations;
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    let marker = directory.path().join("exit-2-at");
    test_inputs::end_with_125_at(&settings.progenitor, solved);
    fs::write(&marker, format!("{task}\n")).unwrap();
    let _stopped = search::search(&settings, &run_directory, &stop, None);
    fs::write(run_directory.join("running").join(leftover), "left\n").unwrap();
    test_inputs::end_with_125_no_more(&run_directory);
    fs::remove_file(&marker).unwrap();
    search::resume(&run_directory, &stop, None).unwrap();
    directory
}

fn resumed_after_a_procreation_stopped() -> TempDir {
    resumed_after_a_stop_at(0, "none", "procreation-1/child/leftover", 1)
}

fn resumed_after_an_evaluation_stopped() -> TempDir {
    resumed_after_a_stop_at(99, "t1", "evaluation-0-t1/output/leftover", 0)
}

fn with_one_evaluation_left_running(directory: &Path) -> PathBuf {
    let mut settings = settings(directory, "spawn.sh");
    settings.initial_procreations = 0;
    settings.workers = 2;
    let run_directory = directory.join("run");
    Workspace::create(&run_directory, &settings.progenitor).unwrap();
    let mut journal = Journal::open(&run_directory).unwrap();
    journal
        .append(&Record::Run {
            format: journal::FORMAT,
            settings,
        })
        .unwrap();
    journal
        .append(&Record::EvaluationStarted {
            harness: 0,
            task: "t1".to_owned(),
        })
        .unwrap();
    run_directory
}

fn records(run_directory: &Path) -> usize {
    fs::read_to_string(journal::path(run_directory))
        .unwrap()
        .lines()
        .count()
}

#[test]
fn what_a_procreation_left_under_running_is_gone_before_it_runs_again() {
    let directory = resumed_after_a_procreation_stopped();
    assert!(!directory.path().join("run/harnesses/1/leftover").exists());
}

#[test]
fn a_procreation_that_runs_again_ends_with_its_child_in_place() {
    let directory = resumed_after_a_procreation_stopped();
    assert!(directory.path().join("run/harnesses/1/solved").is_file());
}

#[test]
fn what_an_evaluation_left_under_running_is_gone_before_it_runs_again() {
    let directory = resumed_after_an_evaluation_stopped();
    assert!(
        !directory
            .path()
            .join("run/evaluations/0/t1/output/leftover")
            .exists()
    );
}

#[test]
fn an_evaluation_that_runs_again_ends_with_its_directory_in_place() {
    let directory = resumed_after_an_evaluation_stopped();
    assert!(
        directory
            .path()
            .join("run/evaluations/0/t1/outcome")
            .is_file()
    );
}

#[test]
fn a_job_that_runs_again_writes_no_second_started_record() {
    let directory = resumed_after_an_evaluation_stopped();
    let journal = fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap();
    assert_eq!(
        journal
            .matches(r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#)
            .count(),
        1
    );
}

#[test]
fn a_new_job_starts_while_a_job_that_runs_again_is_still_running() {
    let directory = test_inputs::temporary_directory();
    let run_directory = with_one_evaluation_left_running(directory.path());
    let stop = AtomicBool::new(false);
    let started = thread::scope(|scope| {
        scope.spawn(|| search::resume(&run_directory, &stop, None));
        let started = test_inputs::within_ten_seconds(|| records(&run_directory) > 2);
        stop.store(true, Ordering::SeqCst);
        started
    });
    assert!(started);
}
