use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::dataset::{Dataset, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Environment, Host, Launch, Program};
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "dataset/t1";
const REPORT: &str = "artifacts = [\"/app/report.txt\"]\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const WRITE_A_DIRECTORY: &str = "mkdir -p out/logs out/cache/inner && echo a > out/a.txt && echo \
                                 b > out/logs/b.txt && touch out/draft.tmp out/logs/run.tmp \
                                 out/cache/inner/entry";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
struct Trialled {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
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
    (
        Environment {
            image: image.name.clone(),
            configuration,
            verifier: None,
        },
        image,
    )
}

fn trialled(task_toml: &str, run_command: &str) -> Trialled {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let dataset = test_inputs::dataset(directory.path(), &["t1"]);
    fs::write(dataset.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_dataset(&Dataset::read(&Disk, &dataset).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", COMPLETE);
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

fn kept(trialled: &Trialled, file: &str) -> String {
    fs::read_to_string(trialled.outputs_directory.trial(0, TASK).join(file)).unwrap()
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

fn artifacts_of(trialled: &Trialled) -> Vec<String> {
    let mut found = Vec::new();
    listed(
        &trialled.outputs_directory.trial(0, TASK).join("artifacts"),
        "",
        &mut found,
    );
    found.sort();
    found
}

fn everything_kept(trialled: &Trialled) -> Vec<String> {
    let mut kept: Vec<String> = fs::read_dir(trialled.outputs_directory.trial(0, TASK))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    kept.sort();
    kept
}

#[test]
fn a_file_the_task_lists_is_kept_at_its_path_without_the_leading_slash() {
    let trialled = trialled(REPORT, WRITE_A_REPORT);
    assert_eq!(kept(&trialled, "artifacts/app/report.txt"), "found\n");
}

#[test]
fn a_directory_the_task_lists_is_kept_with_all_it_holds() {
    let trialled = trialled("artifacts = [\"/app/out/logs\"]\n", WRITE_A_DIRECTORY);
    assert_eq!(
        artifacts_of(&trialled),
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
    let trialled = trialled(
        "[[artifacts]]\nsource = \"/app/out/logs\"\ndestination = \"kept/results\"\n",
        WRITE_A_DIRECTORY,
    );
    assert_eq!(
        artifacts_of(&trialled),
        [
            "kept",
            "kept/results",
            "kept/results/b.txt",
            "kept/results/run.tmp"
        ]
    );
}

#[test]
fn a_directory_the_task_lists_that_no_one_can_write_in_is_kept_with_all_it_holds() {
    let trialled = trialled(
        "artifacts = [\"/app/out\"]\n",
        "mkdir -p out/locked && echo a > out/locked/a.txt && chmod 500 out/locked out",
    );
    assert_eq!(
        artifacts_of(&trialled),
        ["app", "app/out", "app/out/locked", "app/out/locked/a.txt"]
    );
}

#[test]
fn a_directory_in_logs_artifacts_that_no_one_can_write_in_is_kept_with_all_it_holds() {
    let trialled = trialled(
        "schema_version = \"1.4\"\n",
        "mkdir /logs/artifacts/locked && echo a > /logs/artifacts/locked/a.txt && chmod 500 \
         /logs/artifacts/locked",
    );
    assert_eq!(
        artifacts_of(&trialled),
        [
            "logs",
            "logs/artifacts",
            "logs/artifacts/locked",
            "logs/artifacts/locked/a.txt"
        ]
    );
}

#[test]
fn a_directory_no_one_can_read_is_left_out_where_a_pattern_matches_it() {
    let trialled = trialled(
        "[[artifacts]]\nsource = \"/app/out\"\ndestination = \"out\"\nexclude = [\"cache\"]\n",
        "mkdir -p out/cache/inner && touch out/kept out/cache/inner/entry && chmod 300 \
         out/cache/inner out/cache",
    );
    assert_eq!(artifacts_of(&trialled), ["out", "out/kept"]);
}

#[test]
fn nothing_of_the_verifier_is_among_what_is_kept_of_the_paths_the_task_lists() {
    let trialled = trialled(
        "artifacts = [\"/tests\", \"/logs/verifier\"]\n",
        WRITE_A_REPORT,
    );
    assert_eq!(artifacts_of(&trialled), Vec::<String>::new());
}

#[test]
fn a_path_that_is_not_in_the_sandbox_keeps_nothing() {
    let trialled = trialled("artifacts = [\"/app/never/made.txt\"]\n", WRITE_A_REPORT);
    assert!(!everything_kept(&trialled).contains(&"artifacts".to_owned()));
}

#[test]
fn a_symbolic_link_at_a_path_the_task_lists_is_followed() {
    let trialled = trialled(REPORT, "echo found > real.txt && ln -s real.txt report.txt");
    assert!(
        fs::symlink_metadata(
            trialled
                .outputs_directory
                .trial(0, TASK)
                .join("artifacts/app/report.txt")
        )
        .is_ok_and(|report| report.is_file())
    );
}
