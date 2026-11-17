use std::fs;

use cladenc::run_directory::RunDirectory;
use tempfile::TempDir;

fn made() -> (TempDir, RunDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let run_directory = RunDirectory::create(&directory.path().join("run"), &primogenitor).unwrap();
    (directory, run_directory)
}

#[test]
fn the_directory_of_a_build_is_named_after_its_harness_under_builds() {
    let (directory, run_directory) = made();
    assert_eq!(
        run_directory.build(3),
        directory.path().join("run/builds/3")
    );
}

#[test]
fn a_new_run_directory_holds_no_builds() {
    let (directory, _run_directory) = made();
    assert!(!directory.path().join("run/builds").exists());
}

#[test]
fn starting_a_build_returns_its_directory() {
    let (_directory, run_directory) = made();
    assert_eq!(
        run_directory.start_build(0).unwrap(),
        run_directory.build(0)
    );
}

#[test]
fn a_started_build_has_an_empty_directory() {
    let (_directory, run_directory) = made();
    run_directory.start_build(0).unwrap();
    assert_eq!(fs::read_dir(run_directory.build(0)).unwrap().count(), 0);
}

#[test]
fn starting_a_build_again_removes_what_the_earlier_build_left() {
    let (_directory, run_directory) = made();
    run_directory.start_build(0).unwrap();
    fs::write(run_directory.build(0).join("stdout"), "earlier\n").unwrap();
    run_directory.start_build(0).unwrap();
    assert_eq!(fs::read_dir(run_directory.build(0)).unwrap().count(), 0);
}

#[test]
fn a_build_cannot_start_where_a_file_is_in_the_way_of_its_directory() {
    let (directory, run_directory) = made();
    fs::write(directory.path().join("run/builds"), "").unwrap();
    assert!(run_directory.start_build(0).is_err());
}
