use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use crate::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn made() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = directory.path().join("outputs");
    OutputsDirectory::create(&outputs_directory, &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn entries(directory: PathBuf) -> usize {
    fs::read_dir(directory).unwrap().count()
}

#[test]
fn making_an_outputs_directory_where_a_directory_exists_is_refused() {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let refused = OutputsDirectory::create(directory.path(), &primogenitor).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn making_an_outputs_directory_where_a_file_exists_is_refused() {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let taken = directory.path().join("outputs");
    fs::write(&taken, "").unwrap();
    let refused = OutputsDirectory::create(&taken, &primogenitor).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn a_new_outputs_directory_holds_a_copy_of_the_primogenitor_as_harness_0() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        fs::read_to_string(outputs_directory.join("harnesses/0/prompts/system.txt")).unwrap(),
        "Solve the task.\n"
    );
}

#[test]
fn a_new_outputs_directory_holds_an_empty_directory_for_the_attempts_of_harness_0() {
    let (_directory, outputs_directory) = made();
    assert_eq!(entries(outputs_directory.join("attempts/0")), 0);
}

#[test]
fn a_new_outputs_directory_holds_no_procreation() {
    let (_directory, outputs_directory) = made();
    assert_eq!(entries(outputs_directory.join("procreations")), 0);
}

#[test]
fn a_new_outputs_directory_holds_no_benchmark() {
    let (_directory, outputs_directory) = made();
    assert_eq!(entries(outputs_directory.join("benchmarks")), 0);
}

#[test]
fn a_new_outputs_directory_holds_nothing_running() {
    let (_directory, outputs_directory) = made();
    assert_eq!(entries(outputs_directory.join("running")), 0);
}

#[test]
fn a_primogenitor_given_through_a_symbolic_link_is_copied_as_the_directory_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let link = directory.path().join("link");
    symlink(&primogenitor, &link).unwrap();
    let outputs_directory = directory.path().join("outputs");
    OutputsDirectory::create(&outputs_directory, &link).unwrap();
    assert!(
        fs::symlink_metadata(outputs_directory.join("harnesses/0"))
            .unwrap()
            .is_dir()
    );
}

#[test]
fn a_primogenitor_that_does_not_exist_cannot_be_copied() {
    let directory = test_inputs::temporary_directory();
    let missing = directory.path().join("missing");
    let refused =
        OutputsDirectory::create(&directory.path().join("outputs"), &missing).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}

#[test]
fn an_outputs_directory_that_exists_gives_the_same_paths() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        OutputsDirectory::at(&outputs_directory).harness(0),
        outputs_directory.join("harnesses/0")
    );
}
