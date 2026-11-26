use std::io::ErrorKind;
use std::path::PathBuf;

use crate::file_system::tests::double::Memory;
use crate::file_system::{FileSystem, Kind};
use crate::outputs_directory::OutputsDirectory;

fn made() -> (Memory, PathBuf) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = directory.path().join("outputs");
    OutputsDirectory::create(directory.shared(), &outputs_directory, &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn entries(files: &Memory, path: PathBuf) -> usize {
    files.names_in(path).len()
}

#[test]
fn making_an_outputs_directory_where_a_directory_exists_is_refused() {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let refused =
        OutputsDirectory::create(directory.shared(), directory.path(), &primogenitor).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn making_an_outputs_directory_where_a_file_exists_is_refused() {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let taken = directory.path().join("outputs");
    directory.write(&taken, "").unwrap();
    let refused = OutputsDirectory::create(directory.shared(), &taken, &primogenitor).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn a_new_outputs_directory_holds_a_copy_of_the_primogenitor_as_harness_0() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        directory
            .read_to_string(outputs_directory.join("harnesses/0/prompts/system.txt"))
            .unwrap(),
        "Solve the task.\n"
    );
}

#[test]
fn a_new_outputs_directory_holds_an_empty_directory_for_the_trials_of_harness_0() {
    let (directory, outputs_directory) = made();
    assert_eq!(entries(&directory, outputs_directory.join("trials/0")), 0);
}

#[test]
fn a_new_outputs_directory_holds_no_diversification() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        entries(&directory, outputs_directory.join("diversifications")),
        0
    );
}

#[test]
fn a_new_outputs_directory_holds_no_benchmark() {
    let (directory, outputs_directory) = made();
    assert_eq!(entries(&directory, outputs_directory.join("benchmarks")), 0);
}

#[test]
fn a_new_outputs_directory_holds_nothing_running() {
    let (directory, outputs_directory) = made();
    assert_eq!(entries(&directory, outputs_directory.join("running")), 0);
}

#[test]
fn a_primogenitor_given_through_a_symbolic_link_is_copied_as_the_directory_it_leads_to() {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let link = directory.path().join("link");
    directory.symlink(&primogenitor, &link).unwrap();
    let outputs_directory = directory.path().join("outputs");
    OutputsDirectory::create(directory.shared(), &outputs_directory, &link).unwrap();
    assert_eq!(
        directory
            .found(&outputs_directory.join("harnesses/0"))
            .unwrap()
            .kind,
        Kind::Directory
    );
}

#[test]
fn a_primogenitor_that_does_not_exist_cannot_be_copied() {
    let directory = Memory::default();
    let missing = directory.path().join("missing");
    let refused = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &missing,
    )
    .unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}

#[test]
fn an_outputs_directory_that_exists_gives_the_same_paths() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        OutputsDirectory::at(directory.shared(), &outputs_directory).harness(0),
        outputs_directory.join("harnesses/0")
    );
}
