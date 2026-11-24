use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, Environment, GraderImage, Host, Launch, Program};
use cladenc::file_system::Disk;
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

struct Attempted {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
}

struct Attempt<'a> {
    task_toml: &'a str,
    run_command: &'a str,
    grader: &'a str,
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
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), attempt.task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
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
        max_grader_health_check_retries: 0,
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
