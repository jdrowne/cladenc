use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::dataset::Dataset;
use crate::file_system::tests::double::Memory;
use crate::file_system::{FileSystem, Kind};
use crate::outputs_directory::OutputsDirectory;

fn dataset_of(directory: &Path, ids: &[&str]) -> Dataset {
    Dataset {
        directory: directory.to_owned(),
        tasks: ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

fn copied(ids: &[&str]) -> (Memory, PathBuf, io::Result<()>) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2", "t3"]);
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(directory.shared(), &path, &primogenitor).unwrap();
    let result = outputs_directory.copy_dataset(&dataset_of(&dataset, ids));
    (directory, path, result)
}

fn names_in(files: &Memory, path: PathBuf) -> Vec<String> {
    files.names_in(path)
}

#[test]
fn the_copy_of_a_dataset_holds_the_instruction_of_each_task() {
    let (directory, path, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        directory
            .read_to_string(path.join("datasets/dataset/t2/instruction.md"))
            .unwrap(),
        "Finish the work of t2.\n"
    );
}

#[test]
fn the_copy_of_a_dataset_holds_the_environment_of_each_task() {
    let (directory, path, _) = copied(&["t1", "t2", "t3"]);
    assert_eq!(
        directory
            .read_to_string(path.join("datasets/dataset/t3/environment/workspace/work.txt"))
            .unwrap(),
        "to do\n"
    );
}

#[test]
fn the_verifier_of_a_task_stays_executable_in_the_copy() {
    let (directory, path, _) = copied(&["t1", "t2", "t3"]);
    let mode = directory.mode(path.join("datasets/dataset/t1/tests/test.sh"));
    assert_eq!(mode & 0o777, 0o750);
}

#[test]
fn only_the_tasks_that_are_named_are_copied() {
    let (directory, path, _) = copied(&["t1", "t3"]);
    assert_eq!(
        names_in(&directory, path.join("datasets/dataset")),
        ["t1", "t3"]
    );
}

#[test]
fn the_outputs_directory_says_where_the_copy_of_a_task_is() {
    let (directory, path, _) = copied(&["t1"]);
    assert_eq!(
        OutputsDirectory::at(directory.shared(), &path).task("dataset/t1"),
        path.join("datasets/dataset/t1")
    );
}

#[test]
fn a_task_given_through_a_symbolic_link_is_copied_as_the_directory_it_leads_to() {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1"]);
    let elsewhere = test_inputs::task_in(&directory, directory.path(), "kept-elsewhere");
    directory.symlink(elsewhere, dataset.join("t2")).unwrap();
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(directory.shared(), &path, &primogenitor).unwrap();
    outputs_directory
        .copy_dataset(&dataset_of(&dataset, &["t1", "t2"]))
        .unwrap();
    assert_eq!(
        directory
            .found(&path.join("datasets/dataset/t2"))
            .unwrap()
            .kind,
        Kind::Directory
    );
}

#[test]
fn a_task_that_the_dataset_does_not_hold_cannot_be_copied() {
    let (_directory, _, result) = copied(&["t1", "missing"]);
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_dataset_cannot_be_copied_into_an_outputs_directory_that_holds_one_of_its_name() {
    let (directory, path, _) = copied(&["t1"]);
    let again = OutputsDirectory::at(directory.shared(), &path)
        .copy_dataset(&dataset_of(&directory.path().join("dataset"), &["t2"]));
    assert_eq!(again.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

#[test]
fn each_dataset_is_copied_into_a_directory_named_after_it() {
    let (directory, path, _) = copied(&["t1"]);
    let second =
        test_inputs::dataset_named_in(&directory, directory.path(), "refactoring", &["t1"]);
    OutputsDirectory::at(directory.shared(), &path)
        .copy_dataset(&dataset_of(&second, &["t1"]))
        .unwrap();
    assert_eq!(
        names_in(&directory, path.join("datasets")),
        ["dataset", "refactoring"]
    );
}

#[test]
fn a_task_of_the_same_id_in_two_datasets_is_copied_twice() {
    let (directory, path, _) = copied(&["t1"]);
    let second =
        test_inputs::dataset_named_in(&directory, directory.path(), "refactoring", &["t1"]);
    directory
        .write(
            second.join("t1/instruction.md"),
            "Refactor the work of t1.\n",
        )
        .unwrap();
    OutputsDirectory::at(directory.shared(), &path)
        .copy_dataset(&dataset_of(&second, &["t1"]))
        .unwrap();
    assert_eq!(
        ["dataset/t1", "refactoring/t1"].map(|task| {
            directory
                .read_to_string(
                    OutputsDirectory::at(directory.shared(), &path)
                        .task(task)
                        .join("instruction.md"),
                )
                .unwrap()
        }),
        ["Finish the work of t1.\n", "Refactor the work of t1.\n"]
    );
}
