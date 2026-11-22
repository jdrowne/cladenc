use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, GraderImage, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const PUBLISH_A_REPORT: &str = "echo found > /logs/artifacts/report.txt";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const ROWS: &str = "artifacts = [\"/app/rows.txt\"]\n";
const COPY_THE_REPORT: &str = "[[verifier.collect]]\ncommand = \"cat report.txt > rows.txt\"\n";
const NEVER_HEALTHY: &str =
    "[environment.healthcheck]\ncommand = \"exit 1\"\ninterval_sec = 0\nretries = 1\n";

struct Attempted {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

struct Attempt<'a> {
    task_toml: &'a str,
    run_command: &'a str,
    grader: &'a str,
}

fn environment_of(outputs_directory: &OutputsDirectory) -> (Environment, Image) {
    let configuration = TaskConfiguration::read(&outputs_directory.task(TASK)).unwrap();
    let image = test_inputs::image();
    containers::build_environment(
        outputs_directory,
        TASK,
        &image.name,
        configuration.build_time_limit,
        &AtomicBool::new(false),
    )
    .unwrap();
    let grader = configuration
        .grader_environment
        .is_some()
        .then(|| GraderImage {
            name: image.name.clone(),
            holds_grader: false,
        });
    (
        Environment {
            image: image.name.clone(),
            configuration,
            grader,
        },
        image,
    )
}

fn attempted_as(attempt: &Attempt<'_>) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), attempt.task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", attempt.grader);
    let (environment, built) = environment_of(&outputs_directory);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: attempt.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 30,
    };
    let result = containers::run_attempt(
        &outputs_directory,
        &launch,
        containers::Attempt {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &AtomicBool::new(false),
    );
    Attempted {
        _directory: directory,
        _built: built,
        outputs_directory,
        result,
    }
}

fn attempted(task_toml: &str, run_command: &str) -> Attempted {
    attempted_as(&Attempt {
        task_toml,
        run_command,
        grader: COMPLETE,
    })
}

fn kept_at(attempted: &Attempted, file: &str) -> PathBuf {
    attempted.outputs_directory.attempt(0, TASK).join(file)
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(kept_at(attempted, file)).unwrap()
}

fn listed(directory: &Path, below: &str, found: &mut Vec<String>) {
    for entry in fs::read_dir(directory).into_iter().flatten() {
        let entry = entry.unwrap();
        let path = format!("{below}{}", entry.file_name().to_string_lossy());
        if entry.file_type().unwrap().is_dir() {
            listed(&entry.path(), &format!("{path}/"), found);
        }
        found.push(path);
    }
}

fn artifacts_of(attempted: &Attempted) -> Vec<String> {
    let mut found = Vec::new();
    listed(&kept_at(attempted, "artifacts"), "", &mut found);
    found.sort();
    found
}

fn result_of(attempted: &Attempted) -> Option<AttemptResult> {
    attempted.result.as_ref().ok().map(|ended| ended.result)
}

fn collecting(commands: &[&str]) -> String {
    commands
        .iter()
        .map(|command| format!("[[verifier.collect]]\ncommand = \"{command}\"\n\n"))
        .collect()
}

#[test]
fn what_a_harness_left_in_logs_artifacts_is_kept_though_the_task_lists_no_path() {
    let attempted = attempted("", PUBLISH_A_REPORT);
    assert_eq!(
        kept(&attempted, "artifacts/logs/artifacts/report.txt"),
        "found\n"
    );
}

#[test]
fn what_a_harness_left_in_logs_artifacts_is_kept_beside_the_paths_the_task_lists() {
    let attempted = attempted(
        "artifacts = [\"/app/report.txt\"]\n",
        "echo found > report.txt; echo noted > /logs/artifacts/notes.txt",
    );
    assert_eq!(
        artifacts_of(&attempted),
        [
            "app",
            "app/report.txt",
            "logs",
            "logs/artifacts",
            "logs/artifacts/notes.txt"
        ]
    );
}

#[test]
fn a_task_that_lists_logs_artifacts_itself_has_its_own_entry_obeyed_in_its_place() {
    let attempted = attempted(
        "[[artifacts]]\nsource = \"/logs/artifacts\"\ndestination = \"published\"\n",
        PUBLISH_A_REPORT,
    );
    assert_eq!(
        artifacts_of(&attempted),
        ["published", "published/report.txt"]
    );
}

#[test]
fn a_grader_that_runs_separately_finds_what_the_harness_left_in_logs_artifacts() {
    let attempted = attempted_as(&Attempt {
        task_toml: SEPARATE,
        run_command: PUBLISH_A_REPORT,
        grader: "cat /logs/artifacts/report.txt",
    });
    assert_eq!(kept(&attempted, "grader/stdout"), "found\n");
}

#[test]
fn what_a_grader_that_runs_separately_is_given_in_logs_artifacts_is_under_var_log() {
    let attempted = attempted_as(&Attempt {
        task_toml: SEPARATE,
        run_command: PUBLISH_A_REPORT,
        grader: "cat /var/log/cladenc/harbor/artifacts/report.txt",
    });
    assert_eq!(kept(&attempted, "grader/stdout"), "found\n");
}

#[test]
fn a_command_under_collect_is_run_where_the_harness_worked_before_the_paths_are_kept() {
    let attempted = attempted(&format!("{ROWS}\n{COPY_THE_REPORT}"), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "artifacts/app/rows.txt"), "found\n");
}

#[test]
fn what_a_command_under_collect_printed_is_kept_with_the_attempt() {
    let attempted = attempted(&collecting(&["echo collected"]), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "collect/1/stdout"), "collected\n");
}

#[test]
fn what_a_command_under_collect_printed_on_standard_error_is_kept_apart() {
    let attempted = attempted(&collecting(&["echo failed >&2"]), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "collect/1/stderr"), "failed\n");
}

#[test]
fn the_commands_under_collect_are_run_in_the_order_the_task_lists_them() {
    let attempted = attempted(
        &format!(
            "artifacts = [\"/app/order.txt\"]\n\n{}",
            collecting(&["echo first >> order.txt", "echo second >> order.txt"])
        ),
        WRITE_A_REPORT,
    );
    assert_eq!(
        kept(&attempted, "artifacts/app/order.txt"),
        "first\nsecond\n"
    );
}

#[test]
fn what_each_command_under_collect_printed_is_kept_under_its_place_in_the_list() {
    let attempted = attempted(&collecting(&["echo first", "echo second"]), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "collect/2/stdout"), "second\n");
}

#[test]
fn a_command_under_collect_that_fails_does_not_keep_the_next_from_running() {
    let attempted = attempted(&collecting(&["exit 3", "echo after"]), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "collect/2/stdout"), "after\n");
}

#[test]
fn a_command_under_collect_that_fails_does_not_end_the_attempt() {
    let attempted = attempted(&collecting(&["exit 3"]), WRITE_A_REPORT);
    assert_eq!(result_of(&attempted), Some(AttemptResult::Completed));
}

#[test]
fn a_command_under_collect_that_runs_past_its_time_limit_does_not_keep_the_next_from_running() {
    let attempted = attempted(
        "[[verifier.collect]]\ncommand = \"sleep 300\"\ntimeout_sec = 1\n\n[[verifier.collect]]\n\
         command = \"echo after\"\n",
        WRITE_A_REPORT,
    );
    assert_eq!(kept(&attempted, "collect/2/stdout"), "after\n");
}

#[test]
fn a_command_under_collect_is_run_as_the_user_of_the_environment_where_it_names_none() {
    let attempted = attempted(&collecting(&["id -u"]), WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "collect/1/stdout"), "0\n");
}

#[test]
fn a_command_under_collect_is_run_as_the_user_its_entry_names() {
    let attempted = attempted(
        "[[verifier.collect]]\ncommand = \"id -u\"\nuser = 1000\n",
        WRITE_A_REPORT,
    );
    assert_eq!(kept(&attempted, "collect/1/stdout"), "1000\n");
}

#[test]
fn what_a_command_under_collect_made_is_not_among_what_the_harness_left() {
    let attempted = attempted(COPY_THE_REPORT, WRITE_A_REPORT);
    assert!(!kept_at(&attempted, "work/rows.txt").exists());
}

#[test]
fn the_commands_under_collect_are_run_for_a_harness_that_was_stopped() {
    let attempted = attempted(
        &format!("{ROWS}\n[agent]\ntimeout_sec = 2\n\n{COPY_THE_REPORT}"),
        "echo found > report.txt; sleep 300",
    );
    assert_eq!(kept(&attempted, "artifacts/app/rows.txt"), "found\n");
}

#[test]
fn a_command_under_collect_that_exits_with_status_125_is_an_error_of_docker() {
    let attempted = attempted(&collecting(&["exit 125"]), WRITE_A_REPORT);
    assert!(matches!(attempted.result, Err(ContainerError::Docker(_))));
}

#[test]
fn no_command_under_collect_is_run_for_a_harness_that_was_not_started() {
    let attempted = attempted(
        &format!("{NEVER_HEALTHY}\n{COPY_THE_REPORT}"),
        WRITE_A_REPORT,
    );
    assert!(!kept_at(&attempted, "collect").exists());
}

#[test]
fn an_attempt_whose_task_lists_no_command_under_collect_keeps_nothing_of_one() {
    assert!(!kept_at(&attempted("", WRITE_A_REPORT), "collect").exists());
}
