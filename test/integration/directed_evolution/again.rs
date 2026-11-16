use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution;
use cladenc::journal::{self, Journal, Record, Settings};
use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        budget: 4,
        genealogy_growth: 0.6,
        confidence: 99.0,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        assessment_timeout: 60,
        seed: 7,
    }
}

fn resumed_after_a_stop_at(solved: u64, task: Option<&str>, leftover: &str) -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(
        directory.path(),
        test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125,
    );
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    test_inputs::end_with_125_at(&settings.progenitor, solved);
    if let Some(task) = task {
        test_inputs::end_with_125_on(directory.path(), task);
    }
    let _stopped = directed_evolution::start(&settings, &run_directory, &stop, &mut io::sink());
    fs::write(run_directory.join("running").join(leftover), "left\n").unwrap();
    test_inputs::end_with_125_no_more(&run_directory);
    if let Some(task) = task {
        test_inputs::end_with_125_on_no_more(&run_directory, task);
    }
    directed_evolution::resume(&run_directory, &stop, &mut io::sink()).unwrap();
    directory
}

fn resumed_after_a_procreation_stopped() -> TempDir {
    resumed_after_a_stop_at(0, None, "procreation-1/child/leftover")
}

fn resumed_after_an_assessment_stopped() -> TempDir {
    resumed_after_a_stop_at(99, Some("t1"), "assessment-0-benchmark+t1/work/leftover")
}

fn with_one_assessment_left_running(directory: &Path) -> PathBuf {
    let mut settings = settings(
        directory,
        "[ -d /var/log/cladenc/assessments ] || sleep 300",
    );
    settings.max_concurrent_containers = 2;
    let run_directory = directory.join("run");
    Workspace::create(&run_directory, &settings.progenitor)
        .unwrap()
        .copy_benchmark(&settings.benchmarks[0])
        .unwrap();
    let mut journal = Journal::open(&run_directory).unwrap();
    journal
        .append(&Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        })
        .unwrap();
    journal
        .append(&Record::AssessmentStarted {
            harness: 0,
            task: "benchmark/t1".to_owned(),
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
fn what_an_assessment_left_under_running_is_gone_before_it_runs_again() {
    let directory = resumed_after_an_assessment_stopped();
    assert!(
        !directory
            .path()
            .join("run/assessments/0/benchmark/t1/work/leftover")
            .exists()
    );
}

#[test]
fn an_assessment_that_runs_again_ends_with_its_directory_in_place() {
    let directory = resumed_after_an_assessment_stopped();
    assert!(
        directory
            .path()
            .join("run/assessments/0/benchmark/t1/outcome")
            .is_file()
    );
}

#[test]
fn an_action_that_runs_again_writes_no_second_started_record() {
    let directory = resumed_after_an_assessment_stopped();
    let journal = fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap();
    assert_eq!(
        journal
            .matches(r#"{"record":"assessment_started","harness":0,"task":"benchmark/t1"}"#)
            .count(),
        1
    );
}

#[test]
fn a_new_action_starts_while_an_action_that_runs_again_is_still_running() {
    let directory = test_inputs::temporary_directory();
    let run_directory = with_one_assessment_left_running(directory.path());
    let stop = AtomicBool::new(false);
    let started = thread::scope(|scope| {
        scope.spawn(|| directed_evolution::resume(&run_directory, &stop, &mut io::sink()));
        let started = test_inputs::within_ten_seconds(|| records(&run_directory) > 2);
        stop.store(true, Ordering::SeqCst);
        started
    });
    assert!(started);
}
