use std::path::Path;

use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;

const STEPS: &str = "RUN touch /made-here\n";

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

fn started() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    let dockerfile = test_inputs::dockerfile_in(&directory, directory.path(), STEPS);
    outputs_directory.start_image(&dockerfile).unwrap();
    (directory, outputs_directory)
}

#[test]
fn what_is_kept_of_the_image_of_a_directed_evolution_is_in_image_in_the_outputs_directory() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs")).image(),
        Path::new("/outputs/image")
    );
}

#[test]
fn a_new_outputs_directory_keeps_nothing_of_an_image() {
    let (directory, outputs_directory) = made();
    assert!(!directory.exists(&outputs_directory.image()));
}

#[test]
fn starting_the_build_of_an_image_returns_the_directory_that_keeps_it() {
    let (directory, outputs_directory) = made();
    let dockerfile = test_inputs::dockerfile_in(&directory, directory.path(), STEPS);
    assert_eq!(
        outputs_directory.start_image(&dockerfile).unwrap(),
        outputs_directory.image()
    );
}

#[test]
fn starting_the_build_of_an_image_keeps_a_copy_of_its_dockerfile() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        directory
            .read_to_string(outputs_directory.image().join("Dockerfile"))
            .unwrap(),
        format!("FROM {}\n{STEPS}", test_inputs::IMAGE)
    );
}

#[test]
fn starting_the_build_of_an_image_makes_a_build_context_that_holds_nothing() {
    let (directory, outputs_directory) = started();
    assert_eq!(
        directory
            .names_in(outputs_directory.image().join("context"))
            .len(),
        0
    );
}

#[test]
fn the_copy_of_a_dockerfile_is_no_link_to_the_one_that_was_given() {
    let (directory, outputs_directory) = started();
    directory
        .remove_file(directory.path().join("Dockerfile"))
        .unwrap();
    assert!(directory.is_file(&outputs_directory.image().join("Dockerfile")));
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
