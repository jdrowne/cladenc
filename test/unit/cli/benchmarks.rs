use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use cladenc::cli::{Refused, StartOptions};
use cladenc::journal::Settings;
use tempfile::TempDir;

const TAKES: &str = "the directory of a benchmark, in which every directory is a task that holds \
                     a file of text named statement, a directory named files and an executable \
                     file named check";

fn options(directory: &Path) -> StartOptions {
    StartOptions {
        progenitor: test_inputs::progenitor(directory),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark(directory, &["t2", "t1"])],
        docker_image: Some("image".to_owned()),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        run_dir: directory.join("run"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 5,
        workers: 8,
        procreation_timeout: 3600,
        evaluation_timeout: 1800,
        seed: 7,
        quiet: false,
    }
}

fn checked(change: impl FnOnce(&Path, &mut Vec<PathBuf>)) -> (TempDir, Result<Settings, Refused>) {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    change(directory.path(), &mut options.benchmark);
    let settings = options.settings();
    (directory, settings)
}

fn settings_of(change: impl FnOnce(&Path, &mut Vec<PathBuf>)) -> Settings {
    checked(change).1.unwrap()
}

fn reason_of(change: impl FnOnce(&Path, &mut Vec<PathBuf>)) -> (TempDir, String) {
    let (directory, settings) = checked(change);
    (directory, settings.unwrap_err().reason.unwrap_or_default())
}

fn with_a_second(directory: &Path, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(test_inputs::benchmark_named(
        directory,
        "refactoring",
        &["t3", "t1"],
    ));
}

fn with_one_of_the_same_name(directory: &Path, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(test_inputs::benchmark(
        &directory.join("elsewhere"),
        &["t1"],
    ));
}

fn given_twice(_: &Path, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(benchmarks[0].clone());
}

fn missing(directory: &Path, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![directory.join("missing")];
}

fn one_task(_: &Path, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![benchmarks[0].join("t1")];
}

fn with_no_check_in_a_task(_: &Path, benchmarks: &mut Vec<PathBuf>) {
    fs::remove_file(benchmarks[0].join("t2/check")).unwrap();
}

fn with_a_space_in_its_name(directory: &Path, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![test_inputs::benchmark_named(
        directory,
        "my benchmark",
        &["t1"],
    )];
}

fn from_the_working_directory(_: &Path, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![PathBuf::from("test/unit/samples/benchmark")];
}

fn through_a_symbolic_link(directory: &Path, benchmarks: &mut Vec<PathBuf>) {
    let link = directory.join("link");
    symlink(&benchmarks[0], &link).unwrap();
    *benchmarks = vec![link];
}

#[test]
fn a_refused_benchmark_is_refused_by_the_flag_that_names_it() {
    let (_directory, settings) = checked(missing);
    assert_eq!(settings.unwrap_err().option, "--benchmark");
}

#[test]
fn a_refused_benchmark_is_refused_with_what_the_flag_takes() {
    let (_directory, settings) = checked(missing);
    assert_eq!(settings.unwrap_err().accepted, TAKES);
}

#[test]
fn a_benchmark_that_does_not_exist_is_said_as_it_was_given_not_to_be_a_directory() {
    let (directory, reason) = reason_of(missing);
    assert_eq!(
        reason,
        format!(
            "{}: what was given is not a directory that can be read",
            directory.path().join("missing").display()
        )
    );
}

#[test]
fn a_benchmark_that_is_one_task_is_said_to_be_one_task() {
    let (directory, reason) = reason_of(one_task);
    assert_eq!(
        reason,
        format!(
            "{}: what was given is the directory of one task, and a benchmark is the directory \
             that holds the tasks",
            directory.path().join("benchmark/t1").display()
        )
    );
}

#[test]
fn a_benchmark_that_is_refused_for_a_task_names_the_benchmark_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(with_no_check_in_a_task);
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no executable file named check",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_in_a_directory_with_a_space_in_its_name_is_refused_for_its_name() {
    let (directory, reason) = reason_of(with_a_space_in_its_name);
    assert_eq!(
        reason,
        format!(
            "{}: its directory is named `my benchmark`, and the name of a benchmark is made of \
             letters, digits, dots, underscores and hyphens alone",
            directory.path().join("my benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_given_from_the_working_directory_is_made_absolute() {
    assert!(
        settings_of(from_the_working_directory).benchmarks[0]
            .directory
            .is_absolute()
    );
}

#[test]
fn a_task_of_a_search_is_known_by_the_name_of_its_benchmark_and_its_id() {
    assert_eq!(
        settings_of(from_the_working_directory).tasks(),
        ["benchmark/add-a-flag", "benchmark/fix-the-parser"]
    );
}

#[test]
fn a_benchmark_given_through_a_symbolic_link_is_the_directory_it_leads_to() {
    assert!(
        settings_of(through_a_symbolic_link).benchmarks[0]
            .directory
            .ends_with("benchmark")
    );
}

#[test]
fn a_benchmark_given_through_a_symbolic_link_is_named_after_the_directory_it_leads_to() {
    assert_eq!(
        settings_of(through_a_symbolic_link).benchmarks[0].name(),
        "benchmark"
    );
}

#[test]
fn a_search_given_two_benchmarks_has_the_tasks_of_both() {
    assert_eq!(
        settings_of(with_a_second).tasks(),
        [
            "benchmark/t1",
            "benchmark/t2",
            "refactoring/t1",
            "refactoring/t3"
        ]
    );
}

#[test]
fn two_benchmarks_that_each_hold_a_task_of_the_same_id_give_the_search_two_tasks() {
    let tasks = settings_of(with_a_second).tasks();
    assert_eq!(tasks.iter().filter(|task| task.ends_with("/t1")).count(), 2);
}

#[test]
fn the_benchmarks_of_a_search_are_in_the_order_of_their_names_and_not_as_they_were_given() {
    let settings = settings_of(|directory, benchmarks| {
        benchmarks.insert(
            0,
            test_inputs::benchmark_named(directory, "refactoring", &["t3"]),
        );
    });
    assert_eq!(
        settings
            .benchmarks
            .iter()
            .map(|benchmark| benchmark.name())
            .collect::<Vec<_>>(),
        ["benchmark", "refactoring"]
    );
}

#[test]
fn a_benchmark_given_twice_is_refused() {
    let (directory, reason) = reason_of(given_twice);
    assert_eq!(
        reason,
        format!(
            "the benchmark {} is given twice",
            fs::canonicalize(directory.path().join("benchmark"))
                .unwrap()
                .display()
        )
    );
}

#[test]
fn a_benchmark_given_as_itself_and_through_a_symbolic_link_is_given_twice() {
    let (_directory, reason) = reason_of(|directory, benchmarks| {
        let link = directory.join("link");
        symlink(&benchmarks[0], &link).unwrap();
        benchmarks.push(link);
    });
    assert!(reason.ends_with(" is given twice"));
}

#[test]
fn two_benchmarks_in_directories_of_the_same_name_are_refused() {
    let (_directory, reason) = reason_of(with_one_of_the_same_name);
    assert_eq!(
        reason,
        "two of the benchmarks given are in directories named `benchmark`, and a task is known by \
         the name of its benchmark and its id together"
    );
}

#[test]
fn a_second_benchmark_that_cannot_be_used_is_refused_as_the_first_would_be() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        benchmarks.push(directory.join("missing"));
    });
    assert_eq!(
        reason,
        format!(
            "{}: what was given is not a directory that can be read",
            directory.path().join("missing").display()
        )
    );
}
