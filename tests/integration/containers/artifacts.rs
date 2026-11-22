use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const REPORT: &str = "artifacts = [\"/app/report.txt\"]\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const WRITE_A_DIRECTORY: &str = "mkdir -p out/logs out/cache/inner && echo a > out/a.txt && echo \
                                 b > out/logs/b.txt && touch out/draft.tmp out/logs/run.tmp \
                                 out/cache/inner/entry";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
const CHANGE_THE_REPORT: &str = "echo changed > report.txt; echo 1 > /logs/verifier/reward.txt";

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
    (
        Environment {
            image: image.name.clone(),
            configuration,
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

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
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
    listed(
        &attempted
            .outputs_directory
            .attempt(0, TASK)
            .join("artifacts"),
        "",
        &mut found,
    );
    found.sort();
    found
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

#[test]
fn a_file_the_task_lists_is_kept_at_its_path_without_the_leading_slash() {
    let attempted = attempted(REPORT, WRITE_A_REPORT);
    assert_eq!(kept(&attempted, "artifacts/app/report.txt"), "found\n");
}

#[test]
fn a_directory_the_task_lists_is_kept_with_all_it_holds() {
    let attempted = attempted("artifacts = [\"/app/out/logs\"]\n", WRITE_A_DIRECTORY);
    assert_eq!(
        artifacts_of(&attempted),
        [
            "app",
            "app/out",
            "app/out/logs",
            "app/out/logs/b.txt",
            "app/out/logs/run.tmp"
        ]
    );
}

#[test]
fn a_path_the_task_lists_is_kept_at_the_destination_the_task_gives() {
    let attempted = attempted(
        "[[artifacts]]\nsource = \"/app/out/logs\"\ndestination = \"kept/results\"\n",
        WRITE_A_DIRECTORY,
    );
    assert_eq!(
        artifacts_of(&attempted),
        [
            "kept",
            "kept/results",
            "kept/results/b.txt",
            "kept/results/run.tmp"
        ]
    );
}

#[test]
fn what_the_patterns_of_the_task_match_is_left_out_of_a_directory_that_is_kept() {
    let attempted = attempted(
        "[[artifacts]]\nsource = \"/app/out\"\ndestination = \"out\"\nexclude = [\"*.tmp\", \
         \"cache\"]\n",
        WRITE_A_DIRECTORY,
    );
    assert_eq!(
        artifacts_of(&attempted),
        ["out", "out/a.txt", "out/logs", "out/logs/b.txt"]
    );
}

#[test]
fn a_directory_no_one_can_read_is_left_out_where_a_pattern_matches_it() {
    let attempted = attempted(
        "[[artifacts]]\nsource = \"/app/out\"\ndestination = \"out\"\nexclude = [\"cache\"]\n",
        "mkdir -p out/cache/inner && touch out/kept out/cache/inner/entry && chmod 300 \
         out/cache/inner out/cache",
    );
    assert_eq!(artifacts_of(&attempted), ["out", "out/kept"]);
}

#[test]
fn a_path_is_kept_as_the_harness_left_it_and_not_as_the_grader_did() {
    let attempted = attempted_as(&Attempt {
        task_toml: REPORT,
        run_command: WRITE_A_REPORT,
        grader: CHANGE_THE_REPORT,
    });
    assert_eq!(kept(&attempted, "artifacts/app/report.txt"), "found\n");
}

#[test]
fn nothing_of_the_grader_is_among_what_is_kept_of_the_paths_the_task_lists() {
    let attempted = attempted(
        "artifacts = [\"/tests\", \"/logs/verifier\"]\n",
        WRITE_A_REPORT,
    );
    assert_eq!(artifacts_of(&attempted), Vec::<String>::new());
}

#[test]
fn a_path_that_is_not_in_the_container_keeps_nothing() {
    let attempted = attempted("artifacts = [\"/app/never/made.txt\"]\n", WRITE_A_REPORT);
    assert!(!everything_kept(&attempted).contains(&"artifacts".to_owned()));
}

#[test]
fn an_attempt_goes_on_where_a_path_the_task_lists_is_not_in_the_container() {
    let attempted = attempted("artifacts = [\"/app/never/made.txt\"]\n", WRITE_A_REPORT);
    assert_eq!(result_of(&attempted), Some(AttemptResult::Completed));
}

#[test]
fn a_path_listed_after_one_that_is_not_in_the_container_is_kept() {
    let attempted = attempted(
        "artifacts = [\"/app/never/made.txt\", \"/app/report.txt\"]\n",
        WRITE_A_REPORT,
    );
    assert_eq!(artifacts_of(&attempted), ["app", "app/report.txt"]);
}

#[test]
fn an_attempt_whose_task_lists_no_artifacts_keeps_none() {
    let attempted = attempted("", WRITE_A_REPORT);
    assert!(!everything_kept(&attempted).contains(&"artifacts".to_owned()));
}

#[test]
fn a_symbolic_link_at_a_path_the_task_lists_is_followed() {
    let attempted = attempted(REPORT, "echo found > real.txt && ln -s real.txt report.txt");
    assert!(
        fs::symlink_metadata(
            attempted
                .outputs_directory
                .attempt(0, TASK)
                .join("artifacts/app/report.txt")
        )
        .is_ok_and(|report| report.is_file())
    );
}

#[test]
fn what_a_harness_that_was_stopped_left_at_a_path_the_task_lists_is_kept() {
    let attempted = attempted(
        &format!("{REPORT}\n[agent]\ntimeout_sec = 2\n"),
        "echo found > report.txt; sleep 300",
    );
    assert_eq!(kept(&attempted, "artifacts/app/report.txt"), "found\n");
}

#[test]
fn an_attempt_whose_harness_was_not_started_keeps_nothing_of_the_paths_the_task_lists() {
    let attempted = attempted(
        &format!(
            "artifacts = [\"/app/work.txt\"]\n\n[environment.healthcheck]\ncommand = \"exit \
             1\"\ninterval_sec = 0\nretries = 1\n"
        ),
        WRITE_A_REPORT,
    );
    assert_eq!(
        everything_kept(&attempted),
        ["ending", "health-check", "result", "specification"]
    );
}

#[test]
fn the_paths_a_task_lists_under_artifacts_are_read_from_its_task_toml() {
    let directory = test_inputs::temporary_directory();
    fs::write(
        directory.path().join("task.toml"),
        "artifacts = [\"/app/report.txt\", { source = \"/app/out\", destination = \"out\" }]\n",
    )
    .unwrap();
    assert_eq!(
        TaskConfiguration::read(directory.path())
            .unwrap()
            .artifacts
            .len(),
        2
    );
}
