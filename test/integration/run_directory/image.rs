use std::fs;
use std::path::Path;

use cladenc::run_directory::RunDirectory;
use tempfile::TempDir;

const STEPS: &str = "RUN touch /made-here\n";

fn made() -> (TempDir, RunDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let run_directory = RunDirectory::create(&directory.path().join("run"), &primogenitor).unwrap();
    (directory, run_directory)
}

fn started() -> (TempDir, RunDirectory) {
    let (directory, run_directory) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    run_directory.start_image(&dockerfile).unwrap();
    (directory, run_directory)
}

#[test]
fn what_is_kept_of_the_image_of_a_directed_evolution_is_in_image_in_the_run_directory() {
    assert_eq!(
        RunDirectory::at(Path::new("/run")).image(),
        Path::new("/run/image")
    );
}

#[test]
fn a_new_run_directory_keeps_nothing_of_an_image() {
    let (_directory, run_directory) = made();
    assert!(!run_directory.image().exists());
}

#[test]
fn starting_the_build_of_an_image_returns_the_directory_that_keeps_it() {
    let (directory, run_directory) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    assert_eq!(
        run_directory.start_image(&dockerfile).unwrap(),
        run_directory.image()
    );
}

#[test]
fn starting_the_build_of_an_image_keeps_a_copy_of_its_dockerfile() {
    let (_directory, run_directory) = started();
    assert_eq!(
        fs::read_to_string(run_directory.image().join("Dockerfile")).unwrap(),
        format!("FROM {}\n{STEPS}", test_inputs::IMAGE)
    );
}

#[test]
fn starting_the_build_of_an_image_makes_a_build_context_that_holds_nothing() {
    let (_directory, run_directory) = started();
    assert_eq!(
        fs::read_dir(run_directory.image().join("context"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn the_copy_of_a_dockerfile_is_no_link_to_the_one_that_was_given() {
    let (directory, run_directory) = started();
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    assert!(run_directory.image().join("Dockerfile").is_file());
}

#[test]
fn a_dockerfile_that_is_gone_cannot_be_kept() {
    let (directory, run_directory) = made();
    assert!(
        run_directory
            .start_image(&directory.path().join("missing"))
            .is_err()
    );
}
