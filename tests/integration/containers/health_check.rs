use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, HealthCheck, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, HarnessEnding, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "benchmark/t1";
const FAIL_ONCE: &str = "[ -e /tmp/checked ] || { touch /tmp/checked; exit 1; }";
const NEVER_PASSES: &str = "echo not ready; echo no server >&2; exit 1";

struct Attempted {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

fn checked_by(command: &str, more: &str) -> String {
    format!("[environment.healthcheck]\ncommand = '''{command}'''\ninterval_sec = 0.1\n{more}")
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
    (
        Environment {
            image: image.name.clone(),
            configuration,
            grader: None,
        },
        image,
    )
}

fn attempted(task_toml: &str, run_command: &str) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", FINISHED);
    let (environment, built) = environment_of(&outputs_directory);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
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
        &|_| (),
    );
    Attempted {
        _directory: directory,
        _built: built,
        outputs_directory,
        result,
    }
}

fn running(attempted: &Attempted) -> PathBuf {
    attempted.outputs_directory.running_attempt(0, TASK)
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
}

fn everything_kept(attempted: &Attempted) -> Vec<String> {
    let mut kept: Vec<String> = fs::read_dir(attempted.outputs_directory.attempt(0, TASK))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    kept.sort();
    kept
}

fn result_of(attempted: &Attempted) -> Option<AttemptResult> {
    attempted.result.as_ref().ok().map(|ended| ended.result)
}

fn ending_of(attempted: &Attempted) -> Option<HarnessEnding> {
    attempted.result.as_ref().ok().map(|ended| ended.ending)
}

fn printed_by_the_check(task_toml: &str) -> String {
    kept(&attempted(task_toml, FINISH), "health-check/stdout")
}

fn never_passing() -> Attempted {
    attempted(&checked_by(NEVER_PASSES, "retries = 2\n"), FINISH)
}

#[test]
fn the_health_check_of_a_task_is_run_before_its_harness_is_started() {
    let attempted = attempted(&checked_by("touch /checked", ""), "ls /checked");
    assert_eq!(kept(&attempted, "stdout"), "/checked\n");
}

#[test]
fn a_harness_whose_task_passed_its_health_check_is_said_to_have_ended() {
    assert_eq!(
        ending_of(&attempted(&checked_by("true", ""), FINISH)),
        Some(HarnessEnding::Ended)
    );
}

#[test]
fn a_health_check_that_fails_is_run_again_and_the_harness_is_started_once_it_passes() {
    assert_eq!(
        result_of(&attempted(&checked_by(FAIL_ONCE, ""), FINISH)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn a_failure_in_the_start_period_of_a_health_check_is_not_counted() {
    let task_toml = checked_by(
        FAIL_ONCE,
        "retries = 1\nstart_period_sec = 300\nstart_interval_sec = 0.1\n",
    );
    assert_eq!(
        result_of(&attempted(&task_toml, FINISH)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn a_health_check_that_fails_as_many_times_as_its_retries_gives_no_result_but_an_error() {
    assert!(matches!(
        attempted(&checked_by(FAIL_ONCE, "retries = 1\n"), FINISH).result,
        Err(ContainerError::TaskHealthCheck { tries: 1, .. })
    ));
}

#[test]
fn the_error_of_a_harness_that_was_never_started_names_where_what_the_check_printed_is() {
    let attempted = never_passing();
    assert!(matches!(
        &attempted.result,
        Err(ContainerError::TaskHealthCheck { kept, .. })
            if *kept == running(&attempted).join("health-check")
    ));
}

#[test]
fn an_attempt_whose_harness_was_never_started_keeps_no_result() {
    assert!(!never_passing().outputs_directory.attempt(0, TASK).exists());
}

#[test]
fn a_harness_whose_task_did_not_pass_its_health_check_leaves_nothing_as_it_was_never_started() {
    let running = running(&never_passing());
    let left = [
        "stdout",
        "ending",
        "changes",
        "work",
        "collect",
        "artifacts",
    ]
    .map(|left| running.join(left).exists());
    assert_eq!(left, [false; 6]);
}

#[test]
fn what_a_health_check_that_did_not_pass_printed_is_kept_where_the_attempt_was_running() {
    assert_eq!(
        fs::read_to_string(running(&never_passing()).join("health-check/stdout")).unwrap(),
        "not ready\n"
    );
}

#[test]
fn what_a_health_check_that_did_not_pass_printed_on_standard_error_is_kept_as_well() {
    assert_eq!(
        fs::read_to_string(running(&never_passing()).join("health-check/stderr")).unwrap(),
        "no server\n"
    );
}

#[test]
fn what_a_health_check_that_passed_printed_on_standard_error_is_kept_with_the_attempt() {
    let attempted = attempted(&checked_by("echo listening >&2", ""), FINISH);
    assert_eq!(kept(&attempted, "health-check/stderr"), "listening\n");
}

#[test]
fn what_a_health_check_that_passed_printed_is_kept_with_the_attempt() {
    assert_eq!(
        printed_by_the_check(&checked_by("echo ready", "")),
        "ready\n"
    );
}

#[test]
fn an_attempt_whose_task_gives_no_health_check_keeps_nothing_of_one() {
    let attempted = attempted("", FINISH);
    assert!(!everything_kept(&attempted).contains(&"health-check".to_owned()));
}

#[test]
fn a_run_of_a_health_check_that_runs_past_its_time_limit_has_failed() {
    let task_toml = checked_by("sleep 300", "timeout_sec = 1\nretries = 1\n");
    assert!(matches!(
        attempted(&task_toml, FINISH).result,
        Err(ContainerError::TaskHealthCheck { .. })
    ));
}

#[test]
fn a_health_check_runs_as_the_user_of_the_environment_and_not_as_the_user_of_the_harness() {
    let task_toml = format!("[agent]\nuser = 1000\n\n{}", checked_by("id -u", ""));
    assert_eq!(printed_by_the_check(&task_toml), "0\n");
}

#[test]
fn a_health_check_runs_in_the_working_directory_the_task_names() {
    let task_toml = format!(
        "[environment]\nworkdir = \"/srv/elsewhere\"\n\n{}",
        checked_by("pwd", "")
    );
    assert_eq!(printed_by_the_check(&task_toml), "/srv/elsewhere\n");
}

#[test]
fn a_health_check_is_given_the_variables_of_the_environment_of_the_task() {
    let task_toml = format!(
        "{}\n[environment.env]\nMODE = \"strict\"\n",
        checked_by("echo \"$MODE\"", "")
    );
    assert_eq!(printed_by_the_check(&task_toml), "strict\n");
}

#[test]
fn a_health_check_that_exits_with_status_125_is_an_error_of_docker() {
    assert!(matches!(
        attempted(&checked_by("exit 125", ""), FINISH).result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn the_container_of_an_attempt_stays_for_the_longest_its_health_check_can_take_as_well() {
    let task_toml = format!(
        "[agent]\ntimeout_sec = 30\n\n[verifier]\ntimeout_sec = 30\n\n{}",
        checked_by(
            "tr '\\0' ' ' < /proc/1/cmdline",
            "timeout_sec = 10\nretries = 2\n"
        )
    );
    assert_eq!(printed_by_the_check(&task_toml), "sleep 690 ");
}

#[test]
fn the_health_check_of_a_task_is_read_from_its_task_toml() {
    let directory = test_inputs::temporary_directory();
    fs::write(
        directory.path().join("task.toml"),
        checked_by("true", "retries = 4\n"),
    )
    .unwrap();
    assert_eq!(
        TaskConfiguration::read(directory.path())
            .unwrap()
            .health_check
            .map(|HealthCheck { retries, .. }| retries),
        Some(4)
    );
}
