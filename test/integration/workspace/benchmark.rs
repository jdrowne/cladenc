use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn tasks(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

fn copied(ids: &[&str]) -> (TempDir, PathBuf, io::Result<()>) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]);
    let run_directory = directory.path().join("run");
    let workspace = Workspace::create(&run_directory, &progenitor).unwrap();
    let result = workspace.copy_benchmark(&benchmark, &tasks(ids));
    (directory, run_directory, result)
}

#[test]
fn the_copy_of_a_benchmark_holds_the_statement_of_each_task() {
    let (_directory, run_directory, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        fs::read_to_string(run_directory.join("benchmark/t2/statement")).unwrap(),
        "Finish the work of t2.\n"
    );
}

#[test]
fn the_copy_of_a_benchmark_holds_the_files_each_task_starts_from() {
    let (_directory, run_directory, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        fs::read_to_string(run_directory.join("benchmark/t3/files/work.txt")).unwrap(),
        "to do\n"
    );
}

#[test]
fn the_check_of_a_task_stays_executable_in_the_copy() {
    let (_directory, run_directory, _) = copied(&["t1", "t2", "t3"]);
    let mode = fs::metadata(run_directory.join("benchmark/t1/check"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o750);
}

#[test]
fn only_the_tasks_that_are_named_are_copied() {
    let (_directory, run_directory, _) = copied(&["t1", "t3"]);
    assert_eq!(
        fs::read_dir(run_directory.join("benchmark"))
            .unwrap()
            .count(),
        2
    );
}

#[test]
fn the_workspace_says_where_the_copy_of_a_task_is() {
    let (_directory, run_directory, _) = copied(&["t1"]);
    assert_eq!(
        Workspace::at(&run_directory).task("t1"),
        run_directory.join("benchmark/t1")
    );
}

#[test]
fn a_task_given_through_a_symbolic_link_is_copied_as_the_directory_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    let elsewhere = test_inputs::task(directory.path(), "kept-elsewhere");
    symlink(elsewhere, benchmark.join("t2")).unwrap();
    let run_directory = directory.path().join("run");
    let workspace = Workspace::create(&run_directory, &progenitor).unwrap();
    workspace
        .copy_benchmark(&benchmark, &tasks(&["t1", "t2"]))
        .unwrap();
    assert!(
        fs::symlink_metadata(run_directory.join("benchmark/t2"))
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
fn a_benchmark_cannot_be_copied_into_a_run_directory_that_holds_one() {
    let (directory, run_directory, _) = copied(&["t1"]);
    let again = Workspace::at(&run_directory)
        .copy_benchmark(&directory.path().join("benchmark"), &tasks(&["t2"]));
    assert_eq!(again.unwrap_err().kind(), ErrorKind::AlreadyExists);
}
