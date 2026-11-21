use std::fs;
use std::path::Path;

use test_inputs::{benchmark, benchmark_named, temporary_directory};

fn held_by(directory: &Path) -> Vec<String> {
    let mut held: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    held.sort();
    held
}

#[test]
fn a_benchmark_is_the_directory_named_benchmark() {
    let directory = temporary_directory();
    assert_eq!(
        benchmark(directory.path(), &["t1"]),
        directory.path().join("benchmark")
    );
}

#[test]
fn a_benchmark_holds_a_directory_for_each_of_its_tasks() {
    let directory = temporary_directory();
    assert_eq!(
        held_by(&benchmark(directory.path(), &["t2", "t1"])),
        ["t1", "t2"]
    );
}

#[test]
fn a_benchmark_of_no_task_is_an_empty_directory() {
    let directory = temporary_directory();
    assert!(held_by(&benchmark(directory.path(), &[])).is_empty());
}

#[test]
fn each_task_of_a_benchmark_has_an_instruction_its_tests_and_a_workspace() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1", "t2"]);
    assert_eq!(
        held_by(&made.join("t2")),
        ["instruction.md", "tests", "workspace"]
    );
}

#[test]
fn a_benchmark_given_a_name_is_the_directory_of_that_name() {
    let directory = temporary_directory();
    assert_eq!(
        benchmark_named(directory.path(), "refactoring", &["t1"]),
        directory.path().join("refactoring")
    );
}

#[test]
fn a_benchmark_given_a_name_holds_a_directory_for_each_of_its_tasks() {
    let directory = temporary_directory();
    assert_eq!(
        held_by(&benchmark_named(
            directory.path(),
            "refactoring",
            &["t2", "t1"]
        )),
        ["t1", "t2"]
    );
}

#[test]
fn a_benchmark_given_a_name_stands_beside_the_one_named_benchmark() {
    let directory = temporary_directory();
    benchmark(directory.path(), &["t1"]);
    benchmark_named(directory.path(), "refactoring", &["t1"]);
    assert_eq!(held_by(directory.path()), ["benchmark", "refactoring"]);
}
