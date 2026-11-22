use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn benchmark_of(directory: &Path, ids: &[&str]) -> Benchmark {
    Benchmark {
        directory: directory.to_owned(),
        tasks: ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

fn copied(ids: &[&str]) -> (TempDir, PathBuf, io::Result<()>) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]);
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let result = outputs_directory.copy_benchmark(&benchmark_of(&benchmark, ids));
    (directory, path, result)
}

fn names_in(directory: PathBuf) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn the_copy_of_a_benchmark_holds_the_specification_of_each_task() {
    let (_directory, path, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        fs::read_to_string(path.join("benchmarks/benchmark/t2/instruction.md")).unwrap(),
        "Finish the work of t2.\n"
    );
}

#[test]
fn the_copy_of_a_benchmark_holds_the_environment_of_each_task() {
    let (_directory, path, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        fs::read_to_string(path.join("benchmarks/benchmark/t3/environment/workspace/work.txt"))
            .unwrap(),
        "to do\n"
    );
}

#[test]
fn the_grader_of_a_task_stays_executable_in_the_copy() {
    let (_directory, path, _) = copied(&["t1", "t2", "t3"]);
    let mode = fs::metadata(path.join("benchmarks/benchmark/t1/tests/test.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o750);
}

#[test]
fn only_the_tasks_that_are_named_are_copied() {
    let (_directory, path, _) = copied(&["t1", "t3"]);
    assert_eq!(names_in(path.join("benchmarks/benchmark")), ["t1", "t3"]);
}

#[test]
fn the_outputs_directory_says_where_the_copy_of_a_task_is() {
    let (_directory, path, _) = copied(&["t1"]);
    assert_eq!(
        OutputsDirectory::at(&path).task("benchmark/t1"),
        path.join("benchmarks/benchmark/t1")
    );
}

#[test]
fn a_task_given_through_a_symbolic_link_is_copied_as_the_directory_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    let elsewhere = test_inputs::task(directory.path(), "kept-elsewhere");
    symlink(elsewhere, benchmark.join("t2")).unwrap();
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    outputs_directory
        .copy_benchmark(&benchmark_of(&benchmark, &["t1", "t2"]))
        .unwrap();
    assert!(
        fs::symlink_metadata(path.join("benchmarks/benchmark/t2"))
            .unwrap()
            .is_dir()
    );
}

#[test]
fn a_task_that_the_benchmark_does_not_hold_cannot_be_copied() {
    let (_directory, _, result) = copied(&["t1", "missing"]);
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_benchmark_cannot_be_copied_into_an_outputs_directory_that_holds_one_of_its_name() {
    let (directory, path, _) = copied(&["t1"]);
    let again = OutputsDirectory::at(&path)
        .copy_benchmark(&benchmark_of(&directory.path().join("benchmark"), &["t2"]));
    assert_eq!(again.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

#[test]
fn each_benchmark_is_copied_into_a_directory_named_after_it() {
    let (directory, path, _) = copied(&["t1"]);
    let second = test_inputs::benchmark_named(directory.path(), "refactoring", &["t1"]);
    OutputsDirectory::at(&path)
        .copy_benchmark(&benchmark_of(&second, &["t1"]))
        .unwrap();
    assert_eq!(
        names_in(path.join("benchmarks")),
        ["benchmark", "refactoring"]
    );
}

#[test]
fn a_task_of_the_same_id_in_two_benchmarks_is_copied_twice() {
    let (directory, path, _) = copied(&["t1"]);
    let second = test_inputs::benchmark_named(directory.path(), "refactoring", &["t1"]);
    fs::write(
        second.join("t1/instruction.md"),
        "Refactor the work of t1.\n",
    )
    .unwrap();
    OutputsDirectory::at(&path)
        .copy_benchmark(&benchmark_of(&second, &["t1"]))
        .unwrap();
    assert_eq!(
        ["benchmark/t1", "refactoring/t1"].map(|task| {
            fs::read_to_string(
                OutputsDirectory::at(&path)
                    .task(task)
                    .join("instruction.md"),
            )
            .unwrap()
        }),
        ["Finish the work of t1.\n", "Refactor the work of t1.\n"]
    );
}
