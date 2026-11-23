use std::path::Path;

use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "benchmark/t1";

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

#[test]
fn what_is_kept_of_the_environment_of_a_task_is_under_environments_in_the_outputs_directory() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs")).environment(TASK),
        Path::new("/outputs/environments/benchmark/t1")
    );
}

#[test]
fn a_new_outputs_directory_keeps_nothing_of_an_environment() {
    let (directory, outputs_directory) = made();
    assert!(!directory.exists(&outputs_directory.path().join("environments")));
}

#[test]
fn starting_the_build_of_an_environment_returns_the_directory_that_keeps_it() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.start_environment(TASK).unwrap(),
        outputs_directory.environment(TASK)
    );
}

#[test]
fn starting_the_build_of_an_environment_makes_the_directory_that_keeps_it() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_environment(TASK).unwrap();
    assert!(directory.is_dir(&outputs_directory.environment(TASK)));
}

#[test]
fn the_build_of_an_environment_can_be_started_again_over_what_an_earlier_build_kept() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_environment(TASK).unwrap();
    directory
        .write(
            outputs_directory.environment(TASK).join("stdout"),
            "built\n",
        )
        .unwrap();
    assert!(outputs_directory.start_environment(TASK).is_ok());
}

#[test]
fn an_environment_whose_directory_cannot_be_made_cannot_be_kept() {
    let (directory, outputs_directory) = made();
    directory
        .write(outputs_directory.path().join("environments"), "")
        .unwrap();
    assert!(outputs_directory.start_environment(TASK).is_err());
}

#[test]
fn what_is_kept_of_the_environment_of_a_grader_is_in_grader_beside_that_of_its_task() {
    assert_eq!(
        OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
            .grader_environment(TASK),
        Path::new("/outputs/environments/benchmark/t1/grader")
    );
}

#[test]
fn starting_the_build_of_the_environment_of_a_grader_returns_the_directory_that_keeps_it() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.start_grader_environment(TASK).unwrap(),
        outputs_directory.grader_environment(TASK)
    );
}

#[test]
fn starting_the_build_of_the_environment_of_a_grader_makes_the_directory_that_keeps_it() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_grader_environment(TASK).unwrap();
    assert!(directory.is_dir(&outputs_directory.grader_environment(TASK)));
}

#[test]
fn the_build_of_the_environment_of_a_grader_leaves_what_is_kept_of_that_of_its_task() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_environment(TASK).unwrap();
    directory
        .write(
            outputs_directory.environment(TASK).join("stdout"),
            "built\n",
        )
        .unwrap();
    outputs_directory.start_grader_environment(TASK).unwrap();
    assert!(directory.is_file(&outputs_directory.environment(TASK).join("stdout")));
}

#[test]
fn an_environment_of_a_grader_whose_directory_cannot_be_made_cannot_be_kept() {
    let (directory, outputs_directory) = made();
    directory
        .write(outputs_directory.path().join("environments"), "")
        .unwrap();
    assert!(outputs_directory.start_grader_environment(TASK).is_err());
}
