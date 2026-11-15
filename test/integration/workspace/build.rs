use std::fs;

use cladenc::workspace::Workspace;
use tempfile::TempDir;

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

#[test]
fn the_directory_of_a_build_is_named_after_its_harness_under_builds() {
    let (directory, workspace) = made();
    assert_eq!(workspace.build(3), directory.path().join("run/builds/3"));
}

#[test]
fn a_new_run_directory_holds_no_builds() {
    let (directory, _workspace) = made();
    assert!(!directory.path().join("run/builds").exists());
}

#[test]
fn starting_a_build_returns_its_directory() {
    let (_directory, workspace) = made();
    assert_eq!(workspace.start_build(0).unwrap(), workspace.build(0));
}

#[test]
fn a_started_build_has_an_empty_directory() {
    let (_directory, workspace) = made();
    workspace.start_build(0).unwrap();
    assert_eq!(fs::read_dir(workspace.build(0)).unwrap().count(), 0);
}

#[test]
fn starting_a_build_again_removes_what_the_earlier_build_left() {
    let (_directory, workspace) = made();
    workspace.start_build(0).unwrap();
    fs::write(workspace.build(0).join("stdout"), "earlier\n").unwrap();
    workspace.start_build(0).unwrap();
    assert_eq!(fs::read_dir(workspace.build(0)).unwrap().count(), 0);
}

#[test]
fn a_build_cannot_start_where_a_file_is_in_the_way_of_its_directory() {
    let (directory, workspace) = made();
    fs::write(directory.path().join("run/builds"), "").unwrap();
    assert!(workspace.start_build(0).is_err());
}
