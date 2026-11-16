use std::fs;
use std::path::Path;

use cladenc::workspace::Workspace;
use tempfile::TempDir;

const STEPS: &str = "RUN touch /made-here\n";

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &primogenitor).unwrap();
    (directory, workspace)
}

fn started() -> (TempDir, Workspace) {
    let (directory, workspace) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    workspace.start_image(&dockerfile).unwrap();
    (directory, workspace)
}

#[test]
fn what_is_kept_of_the_image_of_a_directed_evolution_is_in_image_in_the_run_directory() {
    assert_eq!(
        Workspace::at(Path::new("/run")).image(),
        Path::new("/run/image")
    );
}

#[test]
fn a_new_run_directory_keeps_nothing_of_an_image() {
    let (_directory, workspace) = made();
    assert!(!workspace.image().exists());
}

#[test]
fn starting_the_build_of_an_image_returns_the_directory_that_keeps_it() {
    let (directory, workspace) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    assert_eq!(
        workspace.start_image(&dockerfile).unwrap(),
        workspace.image()
    );
}

#[test]
fn starting_the_build_of_an_image_keeps_a_copy_of_its_dockerfile() {
    let (_directory, workspace) = started();
    assert_eq!(
        fs::read_to_string(workspace.image().join("Dockerfile")).unwrap(),
        format!("FROM {}\n{STEPS}", test_inputs::IMAGE)
    );
}

#[test]
fn starting_the_build_of_an_image_makes_a_build_context_that_holds_nothing() {
    let (_directory, workspace) = started();
    assert_eq!(
        fs::read_dir(workspace.image().join("context"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn the_copy_of_a_dockerfile_is_no_link_to_the_one_that_was_given() {
    let (directory, workspace) = started();
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    assert!(workspace.image().join("Dockerfile").is_file());
}

#[test]
fn a_dockerfile_that_is_gone_cannot_be_kept() {
    let (directory, workspace) = made();
    assert!(
        workspace
            .start_image(&directory.path().join("missing"))
            .is_err()
    );
}
