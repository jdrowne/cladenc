use std::path::Path;

use crate::dataset::{self, DatasetError};
use crate::file_system::tests::double::Memory;

fn refused(change: impl FnOnce(&Memory, &Path)) -> DatasetError {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2"]);
    change(&directory, &dataset);
    dataset::tasks(&directory, &dataset).unwrap_err()
}

fn no_instruction() -> DatasetError {
    DatasetError::NoInstruction {
        task: "t2".to_owned(),
    }
}

fn no_verifier() -> DatasetError {
    DatasetError::NoVerifier {
        task: "t2".to_owned(),
    }
}

#[test]
fn a_dataset_that_does_not_exist_is_refused() {
    let directory = Memory::default();
    assert_eq!(
        dataset::tasks(&directory, &directory.path().join("missing")),
        Err(DatasetError::NotADirectory)
    );
}

#[test]
fn a_dataset_that_is_a_file_is_refused() {
    let directory = Memory::default();
    let file = directory.path().join("dataset");
    directory.write(&file, "t1\nt2\n").unwrap();
    assert_eq!(
        dataset::tasks(&directory, &file),
        Err(DatasetError::NotADirectory)
    );
}

#[test]
fn a_dataset_that_holds_nothing_is_refused() {
    let directory = Memory::default();
    assert_eq!(
        dataset::tasks(&directory, directory.path()),
        Err(DatasetError::NoTask)
    );
}

#[test]
fn a_dataset_that_holds_files_and_no_directory_is_refused() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("instruction.md"), "Fix the parser.\n")
        .unwrap();
    assert_eq!(
        dataset::tasks(&directory, directory.path()),
        Err(DatasetError::NoTask)
    );
}

#[test]
fn the_directory_of_one_task_is_refused_as_one_task_and_not_a_dataset() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2"]);
    assert_eq!(
        dataset::tasks(&directory, &dataset.join("t1")),
        Err(DatasetError::OneTask)
    );
}

#[test]
fn a_directory_with_a_space_in_its_name_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory.create_dir(dataset.join("task one")).unwrap()),
        DatasetError::NotAnId {
            directory: "task one".to_owned()
        }
    );
}

#[test]
fn a_task_with_no_instruction_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .remove_file(dataset.join("t2/instruction.md"))
            .unwrap()),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_is_in_a_file_named_instruction_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            let task = dataset.join("t2");
            directory
                .rename(task.join("instruction.md"), task.join("instruction"))
                .unwrap();
        }),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t2/instruction.md"))
                .unwrap();
            directory
                .create_dir(dataset.join("t2/instruction.md"))
                .unwrap();
        }),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t2/instruction.md"))
                .unwrap();
            directory
                .symlink("../t1/instruction.md", dataset.join("t2/instruction.md"))
                .unwrap();
        }),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_says_nothing_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .write(dataset.join("t2/instruction.md"), " \n\n")
            .unwrap()),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_is_not_text_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .write(dataset.join("t2/instruction.md"), [0xff, 0xfe])
            .unwrap()),
        no_instruction()
    );
}

#[test]
fn a_task_whose_instruction_holds_a_nul_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .write(dataset.join("t2/instruction.md"), "Fix\0it.\n")
            .unwrap()),
        no_instruction()
    );
}

#[test]
fn a_task_with_no_verifier_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .remove_file(dataset.join("t2/tests/test.sh"))
            .unwrap()),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_not_executable_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .set_mode(dataset.join("t2/tests/test.sh"), 0o640)
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t2/tests/test.sh"))
                .unwrap();
            directory
                .create_dir(dataset.join("t2/tests/test.sh"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_with_no_directory_named_tests_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory.remove_dir_all(dataset.join("t2/tests")).unwrap()),
        no_verifier()
    );
}

#[test]
fn a_task_whose_tests_are_a_symbolic_link_to_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory.remove_dir_all(dataset.join("t2/tests")).unwrap();
            directory
                .symlink("../t1/tests", dataset.join("t2/tests"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_an_executable_file_named_verifier_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .rename(
                    dataset.join("t2/tests/test.sh"),
                    dataset.join("t2/verifier"),
                )
                .unwrap();
        }),
        no_verifier()
    );
}

fn no_environment() -> DatasetError {
    DatasetError::NoEnvironment {
        task: "t2".to_owned(),
    }
}

fn many_containers() -> DatasetError {
    DatasetError::ManyContainers {
        task: "t2".to_owned(),
    }
}

fn with_a_compose_file(name: &str) -> DatasetError {
    refused(|directory, dataset| {
        directory
            .write(dataset.join("t2/environment").join(name), "services: {}\n")
            .unwrap();
    })
}

#[test]
fn a_task_with_no_directory_named_environment_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .remove_dir_all(dataset.join("t2/environment"))
            .unwrap()),
        no_environment()
    );
}

#[test]
fn a_task_whose_environment_holds_no_dockerfile_is_refused() {
    assert_eq!(
        refused(|directory, dataset| directory
            .remove_file(dataset.join("t2/environment/Dockerfile"))
            .unwrap()),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t2/environment/Dockerfile"))
                .unwrap();
            directory
                .create_dir(dataset.join("t2/environment/Dockerfile"))
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t2/environment/Dockerfile"))
                .unwrap();
            directory
                .symlink(
                    "../../t1/environment/Dockerfile",
                    dataset.join("t2/environment/Dockerfile"),
                )
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_environment_is_a_symbolic_link_to_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_dir_all(dataset.join("t2/environment"))
                .unwrap();
            directory
                .symlink("../t1/environment", dataset.join("t2/environment"))
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_named_in_lowercase_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .rename(
                    dataset.join("t2/environment/Dockerfile"),
                    dataset.join("t2/environment/dockerfile"),
                )
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_environment_holds_docker_compose_yaml_is_refused() {
    assert_eq!(
        with_a_compose_file("docker-compose.yaml"),
        many_containers()
    );
}

#[test]
fn a_task_whose_environment_holds_docker_compose_yml_is_refused() {
    assert_eq!(with_a_compose_file("docker-compose.yml"), many_containers());
}

#[test]
fn a_task_whose_environment_holds_compose_yaml_is_refused() {
    assert_eq!(with_a_compose_file("compose.yaml"), many_containers());
}

#[test]
fn a_task_whose_environment_holds_compose_yml_is_refused() {
    assert_eq!(with_a_compose_file("compose.yml"), many_containers());
}

#[test]
fn a_task_whose_compose_file_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .symlink(
                    "Dockerfile",
                    dataset.join("t2/environment/docker-compose.yaml"),
                )
                .unwrap();
        }),
        many_containers()
    );
}

#[test]
fn a_task_with_a_compose_file_beside_its_environment_is_not_refused_for_it() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .write(dataset.join("t2/docker-compose.yaml"), "services: {}\n")
        .unwrap();
    assert_eq!(
        dataset::tasks(&directory, &dataset),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_with_no_verifier_and_no_environment_is_refused_for_its_verifier() {
    assert_eq!(
        refused(|directory, dataset| {
            directory.remove_dir_all(dataset.join("t2/tests")).unwrap();
            directory
                .remove_dir_all(dataset.join("t2/environment"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_that_holds_a_socket_is_refused() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .socket(dataset.join("t2/environment/workspace/socket"))
        .unwrap();
    assert_eq!(
        dataset::tasks(&directory, &dataset),
        Err(DatasetError::OtherFiles {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn the_first_task_that_cannot_be_used_is_the_one_refused() {
    assert_eq!(
        refused(|directory, dataset| {
            directory
                .remove_file(dataset.join("t1/tests/test.sh"))
                .unwrap();
            directory
                .remove_file(dataset.join("t2/instruction.md"))
                .unwrap();
        }),
        DatasetError::NoVerifier {
            task: "t1".to_owned()
        }
    );
}
