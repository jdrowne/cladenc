use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::{self, Journal, Record, Settings};
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&Disk, &test_inputs::benchmark(directory, &["t1"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
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
    let outputs_directory = directory.path().join("outputs");
    test_inputs::end_with_125_at(&settings.primogenitor, solved);
    if let Some(task) = task {
        test_inputs::end_with_125_on(directory.path(), task);
    }
    let _stopped = crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink());
    fs::write(outputs_directory.join("running").join(leftover), "left\n").unwrap();
    test_inputs::end_with_125_no_more(&outputs_directory);
    if let Some(task) = task {
        test_inputs::end_with_125_on_no_more(&outputs_directory, task);
    }
    crate::real::resume(&outputs_directory, &stop, &mut io::sink()).unwrap();
    directory
}

fn resumed_after_a_procreation_stopped() -> TempDir {
    resumed_after_a_stop_at(0, None, "procreation-1/child/leftover")
}

fn resumed_after_an_attempt_stopped() -> TempDir {
    resumed_after_a_stop_at(99, Some("t1"), "attempt-0-benchmark+t1/leftover")
}

fn with_one_attempt_left_running(directory: &Path) -> PathBuf {
    let mut settings = settings(directory, "[ -d /var/log/cladenc/attempts ] || sleep 300");
    settings.max_concurrent_containers = 2;
    test_inputs::task(&settings.benchmarks[0].directory, "t2");
    settings.benchmarks[0].tasks.push("t2".to_owned());
    let outputs_directory = directory.join("outputs");
    OutputsDirectory::create(
        crate::real::disk(),
        &outputs_directory,
        &settings.primogenitor,
    )
    .unwrap()
    .copy_benchmark(&settings.benchmarks[0])
    .unwrap();
    let mut journal = Journal::open(&Disk, &outputs_directory).unwrap();
    journal
        .append(&Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        })
        .unwrap();
    journal
        .append(&Record::AttemptStarted {
            harness: 0,
            task: "benchmark/t1".to_owned(),
        })
        .unwrap();
    outputs_directory
}

fn records(outputs_directory: &Path) -> usize {
    fs::read_to_string(journal::path(outputs_directory))
        .unwrap()
        .lines()
        .count()
}

#[test]
fn what_a_procreation_left_under_running_is_gone_before_it_runs_again() {
    let directory = resumed_after_a_procreation_stopped();
    assert!(
        !directory
            .path()
            .join("outputs/harnesses/1/leftover")
            .exists()
    );
}

#[test]
fn a_procreation_that_runs_again_ends_with_its_child_in_place() {
    let directory = resumed_after_a_procreation_stopped();
    assert!(
        directory
            .path()
            .join("outputs/harnesses/1/solved")
            .is_file()
    );
}

#[test]
fn what_an_attempt_left_under_running_is_gone_before_it_runs_again() {
    let directory = resumed_after_an_attempt_stopped();
    assert!(
        !directory
            .path()
            .join("outputs/attempts/0/benchmark/t1/leftover")
            .exists()
    );
}

#[test]
fn an_attempt_that_runs_again_ends_with_its_directory_in_place() {
    let directory = resumed_after_an_attempt_stopped();
    assert!(
        directory
            .path()
            .join("outputs/attempts/0/benchmark/t1/result")
            .is_file()
    );
}

#[test]
fn an_action_that_runs_again_writes_no_second_started_record() {
    let directory = resumed_after_an_attempt_stopped();
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
    assert_eq!(
        journal
            .matches(r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#)
            .count(),
        1
    );
}

#[test]
fn a_new_action_starts_while_an_action_that_runs_again_is_still_running() {
    let directory = test_inputs::temporary_directory();
    let outputs_directory = with_one_attempt_left_running(directory.path());
    let stop = AtomicBool::new(false);
    let started = thread::scope(|scope| {
        scope.spawn(|| crate::real::resume(&outputs_directory, &stop, &mut io::sink()));
        let started = test_inputs::within_ten_seconds(|| records(&outputs_directory) > 2);
        stop.store(true, Ordering::SeqCst);
        started
    });
    assert!(started);
}
