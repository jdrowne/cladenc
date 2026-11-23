use std::path::{Path, PathBuf};

use crate::cli::StartOptions;
use crate::cli::checks::Refused;
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

const TAKES: &str = "the directory of a benchmark, in which every directory is a task that holds \
                     a file of text named instruction.md, a file named task.toml that is valid as \
                     the task configuration of Harbor, a directory named tests with an executable \
                     file named test.sh in it, or a grader in an image of its own, and a \
                     directory named environment with a file named Dockerfile and no Compose file \
                     in it, or, in place of the Dockerfile, an image named in its task.toml";

fn options(directory: &Memory) -> StartOptions {
    StartOptions {
        primogenitor: test_inputs::primogenitor_in(directory, directory.path()),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark_in(
            directory,
            directory.path(),
            &["t3", "t2", "t1"],
        )],
        procreation_docker_image: Some("image".to_owned()),
        procreation_dockerfile: None,
        procreation_docker_image_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.path().join("outputs"),
        max_procreations: 800,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 30.0,
        corroboration_tasks_percentage: 30.0,
    }
}

fn checked(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> (Memory, Result<Settings, Refused>) {
    let directory = Memory::default();
    let mut options = options(&directory);
    change(&directory, &mut options.benchmark);
    let settings = options.settings(&directory);
    (directory, settings)
}

fn settings_of(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> Settings {
    checked(change).1.unwrap()
}

fn reason_of(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> (Memory, String) {
    let (directory, settings) = checked(change);
    (directory, settings.unwrap_err().reason.unwrap_or_default())
}

fn with_a_second(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(test_inputs::benchmark_named_in(
        directory,
        directory.path(),
        "refactoring",
        &["t3", "t1", "t4"],
    ));
}

fn with_one_of_the_same_name(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(test_inputs::benchmark_in(
        directory,
        &directory.path().join("elsewhere"),
        &["t1"],
    ));
}

fn given_twice(_: &Memory, benchmarks: &mut Vec<PathBuf>) {
    benchmarks.push(benchmarks[0].clone());
}

fn missing(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![directory.path().join("missing")];
}

fn one_task(_: &Memory, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![benchmarks[0].join("t1")];
}

fn with_a_space_in_its_name(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    *benchmarks = vec![test_inputs::benchmark_named_in(
        directory,
        directory.path(),
        "my benchmark",
        &["t1"],
    )];
}

fn from_the_working_directory(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    test_inputs::benchmark_in(
        directory,
        Path::new("/tests/samples"),
        &["add-a-flag", "fix-the-parser", "rename-a-module"],
    );
    *benchmarks = vec![PathBuf::from("tests/samples/benchmark")];
}

fn through_a_symbolic_link(directory: &Memory, benchmarks: &mut Vec<PathBuf>) {
    let link = directory.path().join("link");
    directory.symlink(&benchmarks[0], &link).unwrap();
    *benchmarks = vec![link];
}

#[test]
fn a_refused_benchmark_is_refused_by_the_flag_that_names_it() {
    let (_directory, settings) = checked(missing);
    assert_eq!(settings.unwrap_err().flags, ["--benchmark"]);
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
fn a_benchmark_with_a_task_that_has_no_environment_names_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .remove_dir_all(benchmarks[0].join("t2/environment"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no directory named environment with a file named Dockerfile in \
             it, and its task.toml names no image in its place",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_with_a_task_that_has_no_task_toml_names_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .remove_file(benchmarks[0].join("t2/task.toml"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no file named task.toml",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_with_a_task_whose_task_toml_is_not_valid_names_the_task_the_line_and_why() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .write(
                benchmarks[0].join("t2/task.toml"),
                "[environment]\ncpus = 1\n\n[environment.healthcheck]\nretries = 1.5\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that is not valid at line 5: invalid type: floating \
             point `1.5`, expected i64",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_with_a_task_whose_task_toml_cannot_be_obeyed_names_the_task_and_why() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .write(
                benchmarks[0].join("t2/task.toml"),
                "[agent]\nuser = \"worker\"\n\n[environment]\nworkdir = 3\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that is not valid at line 5: invalid type: integer \
             `3`, expected a string",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_with_a_task_that_asks_for_a_gpu_names_the_task_and_what_it_asks_for() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .write(
                benchmarks[0].join("t2/task.toml"),
                "[environment]\ngpus = 1\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that asks for a GPU, which cladenc has no way to give",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_with_a_task_that_names_an_image_in_its_task_toml_needs_no_dockerfile_for_it() {
    let settings = settings_of(|directory, benchmarks| {
        directory
            .remove_dir_all(benchmarks[0].join("t2/environment"))
            .unwrap();
        directory
            .write(
                benchmarks[0].join("t2/task.toml"),
                "[environment]\ndocker_image = \"debian:stable-slim\"\n",
            )
            .unwrap();
    });
    assert_eq!(
        settings.tasks(),
        ["benchmark/t1", "benchmark/t2", "benchmark/t3"]
    );
}

#[test]
fn a_benchmark_that_is_refused_for_a_task_names_the_benchmark_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, benchmarks| {
        directory
            .remove_dir_all(benchmarks[0].join("t2/tests"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no directory named tests with an executable file named test.sh \
             in it",
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
fn a_task_of_a_directed_evolution_is_known_by_the_name_of_its_benchmark_and_its_id() {
    assert_eq!(
        settings_of(from_the_working_directory).tasks(),
        [
            "benchmark/add-a-flag",
            "benchmark/fix-the-parser",
            "benchmark/rename-a-module"
        ]
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
fn a_directed_evolution_given_two_benchmarks_has_the_tasks_of_both() {
    assert_eq!(
        settings_of(with_a_second).tasks(),
        [
            "benchmark/t1",
            "benchmark/t2",
            "benchmark/t3",
            "refactoring/t1",
            "refactoring/t3",
            "refactoring/t4"
        ]
    );
}

#[test]
fn two_benchmarks_that_each_hold_a_task_of_the_same_id_give_the_directed_evolution_two_tasks() {
    let tasks = settings_of(with_a_second).tasks();
    assert_eq!(tasks.iter().filter(|task| task.ends_with("/t1")).count(), 2);
}

#[test]
fn the_benchmarks_of_a_directed_evolution_are_in_the_order_of_their_names_and_not_as_given() {
    let settings = settings_of(|directory, benchmarks| {
        benchmarks.insert(
            0,
            test_inputs::benchmark_named_in(
                directory,
                directory.path(),
                "refactoring",
                &["t3", "t4", "t5"],
            ),
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
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_benchmark_given_as_itself_and_through_a_symbolic_link_is_given_twice() {
    let (_directory, reason) = reason_of(|directory, benchmarks| {
        let link = directory.path().join("link");
        directory.symlink(&benchmarks[0], &link).unwrap();
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
        benchmarks.push(directory.path().join("missing"));
    });
    assert_eq!(
        reason,
        format!(
            "{}: what was given is not a directory that can be read",
            directory.path().join("missing").display()
        )
    );
}
