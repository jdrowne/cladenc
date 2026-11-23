use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{
    self, Checked, ContainerError, Environment, Host, Launch, Program, Retry,
};
use cladenc::file_system::Disk;
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "benchmark/t1";
const NEVER_READY: &str = "[environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
const NEVER_READY_IN_HALF_A_SECOND: &str = "[environment.healthcheck]\ncommand = \"exit \
                                            1\"\ninterval_sec = 0.5\nretries = 1\n";
const NAMES_ITS_CONTAINER: &str = "[environment.healthcheck]\ncommand = \"cat /etc/hostname; exit \
                                   1\"\ninterval_sec = 0\nretries = 1\n";

struct Prepared {
    directory: TempDir,
    built: Image,
    outputs_directory: OutputsDirectory,
    environment: Environment,
}

struct Most {
    of_the_task: u64,
    of_the_grader: u64,
}

struct Retried {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
    told: Vec<Retry>,
    printed_before_each_retry: Vec<String>,
    containers_at_each_retry: Vec<usize>,
}

fn prepared(task_toml: &str) -> Prepared {
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
    test_inputs::kept_grader(&path, "t1", FINISHED);
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let built = test_inputs::image();
    containers::build_environment(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        TASK,
        &built.name,
        configuration.build_time_limit,
        &AtomicBool::new(false),
    )
    .unwrap();
    let environment = Environment {
        image: built.name.clone(),
        configuration,
        grader: None,
    };
    Prepared {
        directory,
        built,
        outputs_directory,
        environment,
    }
}

fn health_check_of_the_running(outputs_directory: &OutputsDirectory) -> PathBuf {
    outputs_directory
        .running_attempt(0, TASK)
        .join("health-check")
}

fn containers_of(outputs_directory: &OutputsDirectory) -> usize {
    let label = format!(
        "label=cladenc.directed-evolution={}",
        fs::canonicalize(outputs_directory.path())
            .unwrap()
            .display()
    );
    let listed = Command::new("docker")
        .args(["ps", "--quiet", "--filter", &label])
        .output()
        .unwrap();
    String::from_utf8(listed.stdout).unwrap().lines().count()
}

fn retried_as(task_toml: &str, most: &Most) -> Retried {
    let prepared = prepared(task_toml);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: FINISH,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 30,
        max_grader_health_check_retries: most.of_the_grader,
        max_task_health_check_retries: most.of_the_task,
    };
    let printed = health_check_of_the_running(&prepared.outputs_directory).join("stdout");
    let told = RefCell::new(Vec::new());
    let printed_before_each_retry = RefCell::new(Vec::new());
    let containers_at_each_retry = RefCell::new(Vec::new());
    let result = containers::run_attempt(
        Host {
            docker: &Program,
            outputs_directory: &prepared.outputs_directory,
        },
        &launch,
        containers::Attempt {
            harness: 0,
            task: TASK,
            environment: &prepared.environment,
        },
        &AtomicBool::new(false),
        &|retry| {
            told.borrow_mut().push(retry);
            printed_before_each_retry
                .borrow_mut()
                .push(fs::read_to_string(&printed).unwrap_or_default());
            containers_at_each_retry
                .borrow_mut()
                .push(containers_of(&prepared.outputs_directory));
        },
    );
    Retried {
        _directory: prepared.directory,
        _built: prepared.built,
        outputs_directory: prepared.outputs_directory,
        result,
        told: told.into_inner(),
        printed_before_each_retry: printed_before_each_retry.into_inner(),
        containers_at_each_retry: containers_at_each_retry.into_inner(),
    }
}

fn retried(task_toml: &str, max_retries: u64) -> Retried {
    retried_as(
        task_toml,
        &Most {
            of_the_task: max_retries,
            of_the_grader: 0,
        },
    )
}

fn ready_at_the_second_run(interval: u64) -> String {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for answer in ["wait\n", "ready\n"] {
            let _ = listener
                .accept()
                .and_then(|(mut check, _)| check.write_all(answer.as_bytes()));
        }
    });
    format!(
        "[environment.healthcheck]\ncommand = \"bash -c 'read -r answer < \
         /dev/tcp/host.docker.internal/{port} && test $answer = ready'\"\ninterval_sec = \
         {interval}\nretries = 1\n"
    )
}

#[test]
fn the_health_check_of_a_task_is_tried_once_and_then_as_many_times_as_the_most_retries() {
    assert!(matches!(
        retried(NEVER_READY, 2).result,
        Err(ContainerError::TaskHealthCheck { tries: 3, .. })
    ));
}

#[test]
fn the_most_retries_of_the_health_check_of_a_grader_are_not_those_of_that_of_a_task() {
    let most = Most {
        of_the_task: 0,
        of_the_grader: 3,
    };
    assert!(matches!(
        retried_as(NEVER_READY, &most).result,
        Err(ContainerError::TaskHealthCheck { tries: 1, .. })
    ));
}

#[test]
fn the_error_of_a_task_that_never_passed_names_where_what_its_health_check_printed_is() {
    let retried = retried(NEVER_READY, 1);
    assert!(matches!(
        &retried.result,
        Err(ContainerError::TaskHealthCheck { kept, .. })
            if *kept == health_check_of_the_running(&retried.outputs_directory)
    ));
}

#[test]
fn what_the_health_check_of_a_task_printed_in_its_last_try_is_kept() {
    let retried = retried(NEVER_READY, 1);
    assert_eq!(
        fs::read_to_string(health_check_of_the_running(&retried.outputs_directory).join("stdout"))
            .unwrap(),
        "not ready\n"
    );
}

#[test]
fn an_attempt_whose_task_passed_its_health_check_in_no_try_keeps_no_result() {
    assert!(
        !retried(NEVER_READY, 1)
            .outputs_directory
            .attempt(0, TASK)
            .exists()
    );
}

#[test]
fn each_retry_of_a_task_is_told_with_a_wait_that_starts_at_the_interval_and_doubles() {
    assert_eq!(
        retried(NEVER_READY_IN_HALF_A_SECOND, 2).told,
        [
            Retry {
                checked: Checked::Task,
                number: 1,
                most: 2,
                wait: Duration::from_millis(500),
            },
            Retry {
                checked: Checked::Task,
                number: 2,
                most: 2,
                wait: Duration::from_secs(1),
            }
        ]
    );
}

#[test]
fn no_retry_of_a_task_is_told_where_the_most_retries_are_0() {
    assert_eq!(retried(NEVER_READY, 0).told, Vec::<Retry>::new());
}

#[test]
fn each_retry_of_the_health_check_of_a_task_is_in_a_new_container() {
    let retried = retried(NAMES_ITS_CONTAINER, 2);
    let last =
        fs::read_to_string(health_check_of_the_running(&retried.outputs_directory).join("stdout"))
            .unwrap();
    let containers: BTreeSet<String> = retried
        .printed_before_each_retry
        .into_iter()
        .chain([last])
        .collect();
    assert_eq!(containers.len(), 3);
}

#[test]
fn the_container_of_a_task_is_removed_before_the_wait_for_a_retry() {
    assert_eq!(retried(NEVER_READY, 2).containers_at_each_retry, [0, 0]);
}

#[test]
fn a_harness_whose_task_passes_its_health_check_in_a_container_brought_up_again_is_started() {
    assert_eq!(
        retried(&ready_at_the_second_run(0), 1)
            .result
            .ok()
            .map(|attempted| attempted.result),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn a_health_check_of_a_task_that_passes_at_a_retry_is_not_tried_again() {
    assert_eq!(retried(&ready_at_the_second_run(0), 3).told.len(), 1);
}

#[test]
fn the_seconds_of_an_attempt_leave_out_the_tries_that_did_not_pass_and_the_wait_after_them() {
    assert!(
        retried(&ready_at_the_second_run(20), 1)
            .result
            .is_ok_and(|attempted| attempted.seconds < 20.0)
    );
}
