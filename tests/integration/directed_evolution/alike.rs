use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::cli::Cli;
use cladenc::directed_evolution;
use cladenc::journal::{self, Settings};
use cladenc::report;
use clap::CommandFactory;
use tempfile::TempDir;

fn run_command_the_help_gives(command: &str) -> String {
    let mut cladenc = Cli::command();
    let help = cladenc
        .find_subcommand_mut("start")
        .unwrap()
        .render_long_help()
        .to_string();
    help.split_once("--harness-run-command '")
        .and_then(|(_, after)| after.split_once('\''))
        .map(|(given, _)| given.replace("<COMMAND>", command))
        .unwrap()
}

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(directory, &["t1", "t2", "t3"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command_the_help_gives("echo attempted > attempted"),
        max_procreations: 2,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 33.5,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 33.5,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    directory
}

fn kept_in(tree: &Path) -> Vec<String> {
    let mut kept: Vec<String> = fs::read_dir(tree)
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() && !path.is_symlink() {
                kept_in(&path)
                    .into_iter()
                    .map(|inside| format!("{name}/{inside}"))
                    .collect()
            } else {
                vec![format!("{name}: {}", fs::read_to_string(&path).unwrap())]
            }
        })
        .collect();
    kept.sort();
    kept
}

fn source_code_of(directory: &TempDir, harness: &str) -> Vec<String> {
    kept_in(&directory.path().join("outputs/harnesses").join(harness))
}

fn source_code_without_the_file_the_run_command_adds_to(
    directory: &TempDir,
    harness: &str,
) -> Vec<String> {
    source_code_of(directory, harness)
        .into_iter()
        .filter(|kept| !kept.starts_with("procreated: "))
        .collect()
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap()
}

#[test]
fn the_run_command_the_help_gives_starts_the_command_of_the_other_only_for_an_attempt() {
    assert_eq!(
        run_command_the_help_gives("sh harness.sh"),
        "if [ -d /var/log/cladenc/attempts ]\n        then echo >> procreated; else sh harness.sh; \
         fi"
    );
}

#[test]
fn with_the_run_command_the_help_gives_every_procreation_creates_its_child() {
    let directory = evolved();
    assert_eq!(
        journal_of(&directory)
            .lines()
            .filter(|line| line.contains(r#""record":"procreation_finished""#))
            .filter(|line| line.contains(r#""outcome":"created""#))
            .count(),
        2
    );
}

#[test]
fn with_the_run_command_the_help_gives_each_child_holds_the_files_of_the_primogenitor_as_they_were()
{
    let directory = evolved();
    assert_eq!(
        ["1", "2"]
            .map(|child| source_code_without_the_file_the_run_command_adds_to(&directory, child)),
        [
            source_code_of(&directory, "0"),
            source_code_of(&directory, "0")
        ]
    );
}

#[test]
fn with_the_run_command_the_help_gives_a_child_of_the_primogenitor_holds_a_file_of_one_line_more() {
    let directory = evolved();
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/1/procreated")).unwrap(),
        "\n"
    );
}

#[test]
fn with_the_run_command_the_help_gives_every_harness_is_benchmarked_in_full() {
    let directory = evolved();
    let journal = journal_of(&directory);
    assert_eq!(
        [0, 1, 2].map(|harness| {
            journal
                .matches(&format!(
                    r#""record":"attempt_finished","harness":{harness},"#
                ))
                .count()
        }),
        [3, 3, 3]
    );
}

#[test]
fn with_the_run_command_the_help_gives_a_harness_is_started_as_ever_for_an_attempt() {
    let directory = evolved();
    assert_eq!(
        fs::read_to_string(
            directory
                .path()
                .join("outputs/attempts/2/benchmark/t1/work/attempted")
        )
        .unwrap(),
        "attempted\n"
    );
}

#[test]
fn the_report_of_such_a_directed_evolution_counts_the_tasks_one_harness_at_least_completed() {
    let directory = evolved();
    let report = report::read(&directory.path().join("outputs")).unwrap();
    assert_eq!(
        (
            report.selection_tasks_completed_by_any,
            report.selection_tasks,
            report.corroboration_tasks_completed_by_any,
            report.corroboration_tasks,
            report.non_holdout_tasks_completed_by_any,
            report.non_holdout_tasks
        ),
        (0, 1, 0, 1, 0, 1)
    );
}
