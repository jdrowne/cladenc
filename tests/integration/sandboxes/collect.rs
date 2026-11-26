use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Environment, Host, Launch, Program, VerifierImage};
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const PUBLISH_A_REPORT: &str = "echo found > /logs/artifacts/report.txt";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const ROWS: &str = "artifacts = [\"/app/rows.txt\"]\n";
const COPY_THE_REPORT: &str = "[[verifier.collect]]\ncommand = \"cat report.txt > rows.txt\"\n";

struct Trialled {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
}

struct Trial<'a> {
    task_toml: &'a str,
    run_command: &'a str,
    verifier: &'a str,
}

fn environment_of(outputs_directory: &OutputsDirectory) -> (Environment, Image) {
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let image = test_inputs::image();
    sandboxes::build_environment(
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
    let verifier = configuration
        .verifier_environment
        .is_some()
        .then(|| VerifierImage {
            name: image.name.clone(),
            holds_verifier: false,
        });
    (
        Environment {
            image: image.name.clone(),
            configuration,
            verifier,
        },
        image,
    )
}

fn trialled_as(trial: &Trial<'_>) -> Trialled {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), trial.task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", trial.verifier);
    let (environment, built) = environment_of(&outputs_directory);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: trial.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let _ = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        sandboxes::Trial {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &AtomicBool::new(false),
        &|_| (),
    );
    Trialled {
        _directory: directory,
        _built: built,
        outputs_directory,
    }
}

fn trialled(task_toml: &str, run_command: &str) -> Trialled {
    trialled_as(&Trial {
        task_toml,
        run_command,
        verifier: COMPLETE,
    })
}

fn kept_at(trialled: &Trialled, file: &str) -> PathBuf {
    trialled.outputs_directory.trial(0, TASK).join(file)
}

fn kept(trialled: &Trialled, file: &str) -> String {
    fs::read_to_string(kept_at(trialled, file)).unwrap()
}

fn collecting(commands: &[&str]) -> String {
    commands
        .iter()
        .map(|command| format!("[[verifier.collect]]\ncommand = \"{command}\"\n\n"))
        .collect()
}

#[test]
fn what_a_harness_left_in_logs_artifacts_is_kept_though_the_task_lists_no_path() {
    let trialled = trialled("", PUBLISH_A_REPORT);
    assert_eq!(
        kept(&trialled, "artifacts/logs/artifacts/report.txt"),
        "found\n"
    );
}

#[test]
fn a_verifier_that_runs_separately_finds_what_the_harness_left_in_logs_artifacts() {
    let trialled = trialled_as(&Trial {
        task_toml: SEPARATE,
        run_command: PUBLISH_A_REPORT,
        verifier: "cat /logs/artifacts/report.txt",
    });
    assert_eq!(kept(&trialled, "verifier/stdout"), "found\n");
}

#[test]
fn what_a_verifier_that_runs_separately_is_given_in_logs_artifacts_is_under_var_log() {
    let trialled = trialled_as(&Trial {
        task_toml: SEPARATE,
        run_command: PUBLISH_A_REPORT,
        verifier: "cat /var/log/cladenc/harbor/artifacts/report.txt",
    });
    assert_eq!(kept(&trialled, "verifier/stdout"), "found\n");
}

#[test]
fn a_command_under_collect_is_run_where_the_harness_worked_before_the_paths_are_kept() {
    let trialled = trialled(&format!("{ROWS}\n{COPY_THE_REPORT}"), WRITE_A_REPORT);
    assert_eq!(kept(&trialled, "artifacts/app/rows.txt"), "found\n");
}

#[test]
fn a_command_under_collect_is_run_as_the_user_of_the_environment_where_it_names_none() {
    let trialled = trialled(&collecting(&["id -u"]), WRITE_A_REPORT);
    assert_eq!(kept(&trialled, "collection-hooks/1/stdout"), "0\n");
}

#[test]
fn a_command_under_collect_is_run_as_the_user_its_entry_names() {
    let trialled = trialled(
        "[[verifier.collect]]\ncommand = \"id -u\"\nuser = 1000\n",
        WRITE_A_REPORT,
    );
    assert_eq!(kept(&trialled, "collection-hooks/1/stdout"), "1000\n");
}
