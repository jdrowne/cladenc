use std::fs;
use std::path::Path;

use crate::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const STEPS: &str = "RUN touch /made-here\n";

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn started() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    outputs_directory.start_image(&dockerfile).unwrap();
    (directory, outputs_directory)
}

#[test]
fn what_is_kept_of_the_image_of_a_directed_evolution_is_in_image_in_the_outputs_directory() {
    assert_eq!(
        OutputsDirectory::at(Path::new("/outputs")).image(),
        Path::new("/outputs/image")
    );
}

#[test]
fn a_new_outputs_directory_keeps_nothing_of_an_image() {
    let (_directory, outputs_directory) = made();
    assert!(!outputs_directory.image().exists());
}

#[test]
fn starting_the_build_of_an_image_returns_the_directory_that_keeps_it() {
    let (directory, outputs_directory) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), STEPS);
    assert_eq!(
        outputs_directory.start_image(&dockerfile).unwrap(),
        outputs_directory.image()
    );
}

#[test]
fn starting_the_build_of_an_image_keeps_a_copy_of_its_dockerfile() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        fs::read_to_string(outputs_directory.image().join("Dockerfile")).unwrap(),
        format!("FROM {}\n{STEPS}", test_inputs::IMAGE)
    );
}

#[test]
fn starting_the_build_of_an_image_makes_a_build_context_that_holds_nothing() {
    let (_directory, outputs_directory) = started();
    assert_eq!(
        fs::read_dir(outputs_directory.image().join("context"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn the_copy_of_a_dockerfile_is_no_link_to_the_one_that_was_given() {
    let (directory, outputs_directory) = started();
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    assert!(outputs_directory.image().join("Dockerfile").is_file());
}

#[test]
fn a_dockerfile_that_is_gone_cannot_be_kept() {
    let (directory, outputs_directory) = made();
    assert!(
        outputs_directory
            .start_image(&directory.path().join("missing"))
            .is_err()
    );
}
