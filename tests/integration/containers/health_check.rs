use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, Environment, Host, Launch, Program};
use cladenc::file_system::Disk;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "benchmark/t1";
struct Attempted {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
}

fn checked_by(command: &str, more: &str) -> String {
    format!("[environment.healthcheck]\ncommand = '''{command}'''\ninterval_sec = 0.1\n{more}")
}

fn environment_of(outputs_directory: &OutputsDirectory) -> (Environment, Image) {
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let image = test_inputs::image();
    containers::build_environment(
        Host {
            docker: &Program,
            outputs_directory,
        },
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
            verifier: None,
        },
        image,
    )
}

fn attempted(task_toml: &str, run_command: &str) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", FINISHED);
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
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let _ = containers::run_attempt(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
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
    }
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
}

fn printed_by_the_check(task_toml: &str) -> String {
    kept(&attempted(task_toml, FINISH), "health-check/stdout")
}

#[test]
fn the_health_check_of_a_task_is_run_before_its_harness_is_started() {
    let attempted = attempted(&checked_by("touch /checked", ""), "ls /checked");
    assert_eq!(kept(&attempted, "stdout"), "/checked\n");
}

#[test]
fn what_a_health_check_that_passed_printed_is_kept_with_the_attempt() {
    assert_eq!(
        printed_by_the_check(&checked_by("echo ready", "")),
        "ready\n"
    );
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
