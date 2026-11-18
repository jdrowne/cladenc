use std::fs;

use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

#[test]
fn the_directory_of_a_build_is_named_after_its_harness_under_builds() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.build(3),
        directory.path().join("outputs/builds/3")
    );
}

#[test]
fn a_new_outputs_directory_holds_no_builds() {
    let (directory, _outputs_directory) = made();
    assert!(!directory.path().join("outputs/builds").exists());
}

#[test]
fn starting_a_build_returns_its_directory() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.start_build(0).unwrap(),
        outputs_directory.build(0)
    );
}

#[test]
fn a_started_build_has_an_empty_directory() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_build(0).unwrap();
    assert_eq!(fs::read_dir(outputs_directory.build(0)).unwrap().count(), 0);
}

#[test]
fn starting_a_build_again_removes_what_the_earlier_build_left() {
    let (_directory, outputs_directory) = made();
    outputs_directory.start_build(0).unwrap();
    fs::write(outputs_directory.build(0).join("stdout"), "earlier\n").unwrap();
    outputs_directory.start_build(0).unwrap();
    assert_eq!(fs::read_dir(outputs_directory.build(0)).unwrap().count(), 0);
}

#[test]
fn a_build_cannot_start_where_a_file_is_in_the_way_of_its_directory() {
    let (directory, outputs_directory) = made();
    fs::write(directory.path().join("outputs/builds"), "").unwrap();
    assert!(outputs_directory.start_build(0).is_err());
}
