use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::cli::Cli;
use cladenc::directed_evolution;
use cladenc::journal::{self, Settings};
use clap::CommandFactory;
use tempfile::TempDir;

fn run_command_the_help_gives(command: &str) -> String {
    let mut cladenc = Cli::command();
    let help = cladenc
        .find_subcommand_mut("start")
        .unwrap()
        .render_long_help()
        .to_string();
    help.lines()
        .find(|line| {
            line.trim_start()
                .starts_with("'[ -d /var/log/cladenc/attempts ]")
        })
        .unwrap()
        .trim()
        .trim_matches('\'')
        .replace("<COMMAND>", command)
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
        genealogy_growth: 0.0,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        attempt_timeout: 30,
        holdout_percentage: 33.0,
        holdout_tasks: vec!["benchmark/t2".to_owned()],
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

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap()
}

#[test]
fn the_run_command_the_help_gives_starts_the_command_of_the_other_only_for_an_attempt() {
    assert_eq!(
        run_command_the_help_gives("sh harness.sh"),
        "[ -d /var/log/cladenc/attempts ] && exit 0; sh harness.sh"
    );
}

#[test]
fn with_the_run_command_the_help_gives_every_procreation_creates_its_child() {
    let directory = evolved();
    assert_eq!(
        journal_of(&directory)
            .matches(r#""outcome":"created""#)
            .count(),
        2
    );
}

#[test]
fn with_the_run_command_the_help_gives_the_source_code_of_each_child_is_that_of_the_primogenitor() {
    let directory = evolved();
    assert_eq!(
        ["1", "2"].map(|child| source_code_of(&directory, child)),
        [
            source_code_of(&directory, "0"),
            source_code_of(&directory, "0")
        ]
    );
}

#[test]
fn with_the_run_command_the_help_gives_every_harness_is_benchmarked_in_full() {
    let directory = evolved();
    assert_eq!(
        journal_of(&directory)
            .matches(r#""record":"attempt_finished""#)
            .count(),
        9
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
