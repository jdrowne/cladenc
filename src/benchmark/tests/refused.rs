use std::path::Path;

use crate::benchmark::{self, BenchmarkError};
use crate::file_system::tests::double::Memory;

fn refused(change: impl FnOnce(&Memory, &Path)) -> BenchmarkError {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    change(&directory, &benchmark);
    benchmark::tasks(&directory, &benchmark).unwrap_err()
}

fn no_specification() -> BenchmarkError {
    BenchmarkError::NoSpecification {
        task: "t2".to_owned(),
    }
}

fn no_verifier() -> BenchmarkError {
    BenchmarkError::NoVerifier {
        task: "t2".to_owned(),
    }
}

#[test]
fn a_benchmark_that_does_not_exist_is_refused() {
    let directory = Memory::default();
    assert_eq!(
        benchmark::tasks(&directory, &directory.path().join("missing")),
        Err(BenchmarkError::NotADirectory)
    );
}

#[test]
fn a_benchmark_that_is_a_file_is_refused() {
    let directory = Memory::default();
    let file = directory.path().join("benchmark");
    directory.write(&file, "t1\nt2\n").unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &file),
        Err(BenchmarkError::NotADirectory)
    );
}

#[test]
fn a_benchmark_that_holds_nothing_is_refused() {
    let directory = Memory::default();
    assert_eq!(
        benchmark::tasks(&directory, directory.path()),
        Err(BenchmarkError::NoTask)
    );
}

#[test]
fn a_benchmark_that_holds_files_and_no_directory_is_refused() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("instruction.md"), "Fix the parser.\n")
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, directory.path()),
        Err(BenchmarkError::NoTask)
    );
}

#[test]
fn the_directory_of_one_task_is_refused_as_one_task_and_not_a_benchmark() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    assert_eq!(
        benchmark::tasks(&directory, &benchmark.join("t1")),
        Err(BenchmarkError::OneTask)
    );
}

#[test]
fn a_directory_with_a_space_in_its_name_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory.create_dir(benchmark.join("task one")).unwrap()),
        BenchmarkError::NotAnId {
            directory: "task one".to_owned()
        }
    );
}

#[test]
fn a_task_with_no_specification_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .remove_file(benchmark.join("t2/instruction.md"))
            .unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_in_a_file_named_specification_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            let task = benchmark.join("t2");
            directory
                .rename(task.join("instruction.md"), task.join("specification"))
                .unwrap();
        }),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t2/instruction.md"))
                .unwrap();
            directory
                .create_dir(benchmark.join("t2/instruction.md"))
                .unwrap();
        }),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t2/instruction.md"))
                .unwrap();
            directory
                .symlink("../t1/instruction.md", benchmark.join("t2/instruction.md"))
                .unwrap();
        }),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_says_nothing_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .write(benchmark.join("t2/instruction.md"), " \n\n")
            .unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_not_text_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .write(benchmark.join("t2/instruction.md"), [0xff, 0xfe])
            .unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_holds_a_nul_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .write(benchmark.join("t2/instruction.md"), "Fix\0it.\n")
            .unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_with_no_verifier_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .remove_file(benchmark.join("t2/tests/test.sh"))
            .unwrap()),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_not_executable_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .set_mode(benchmark.join("t2/tests/test.sh"), 0o640)
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t2/tests/test.sh"))
                .unwrap();
            directory
                .create_dir(benchmark.join("t2/tests/test.sh"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_with_no_directory_named_tests_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .remove_dir_all(benchmark.join("t2/tests"))
            .unwrap()),
        no_verifier()
    );
}

#[test]
fn a_task_whose_tests_are_a_symbolic_link_to_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_dir_all(benchmark.join("t2/tests"))
                .unwrap();
            directory
                .symlink("../t1/tests", benchmark.join("t2/tests"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_whose_verifier_is_an_executable_file_named_verifier_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .rename(
                    benchmark.join("t2/tests/test.sh"),
                    benchmark.join("t2/verifier"),
                )
                .unwrap();
        }),
        no_verifier()
    );
}

fn no_environment() -> BenchmarkError {
    BenchmarkError::NoEnvironment {
        task: "t2".to_owned(),
    }
}

fn many_containers() -> BenchmarkError {
    BenchmarkError::ManyContainers {
        task: "t2".to_owned(),
    }
}

fn with_a_compose_file(name: &str) -> BenchmarkError {
    refused(|directory, benchmark| {
        directory
            .write(
                benchmark.join("t2/environment").join(name),
                "services: {}\n",
            )
            .unwrap();
    })
}

#[test]
fn a_task_with_no_directory_named_environment_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .remove_dir_all(benchmark.join("t2/environment"))
            .unwrap()),
        no_environment()
    );
}

#[test]
fn a_task_whose_environment_holds_no_dockerfile_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| directory
            .remove_file(benchmark.join("t2/environment/Dockerfile"))
            .unwrap()),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t2/environment/Dockerfile"))
                .unwrap();
            directory
                .create_dir(benchmark.join("t2/environment/Dockerfile"))
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t2/environment/Dockerfile"))
                .unwrap();
            directory
                .symlink(
                    "../../t1/environment/Dockerfile",
                    benchmark.join("t2/environment/Dockerfile"),
                )
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_environment_is_a_symbolic_link_to_a_directory_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_dir_all(benchmark.join("t2/environment"))
                .unwrap();
            directory
                .symlink("../t1/environment", benchmark.join("t2/environment"))
                .unwrap();
        }),
        no_environment()
    );
}

#[test]
fn a_task_whose_dockerfile_is_named_in_lowercase_is_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .rename(
                    benchmark.join("t2/environment/Dockerfile"),
                    benchmark.join("t2/environment/dockerfile"),
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
        refused(|directory, benchmark| {
            directory
                .symlink(
                    "Dockerfile",
                    benchmark.join("t2/environment/docker-compose.yaml"),
                )
                .unwrap();
        }),
        many_containers()
    );
}

#[test]
fn a_task_with_a_compose_file_beside_its_environment_is_not_refused_for_it() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .write(benchmark.join("t2/docker-compose.yaml"), "services: {}\n")
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_with_no_verifier_and_no_environment_is_refused_for_its_verifier() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_dir_all(benchmark.join("t2/tests"))
                .unwrap();
            directory
                .remove_dir_all(benchmark.join("t2/environment"))
                .unwrap();
        }),
        no_verifier()
    );
}

#[test]
fn a_task_that_holds_a_socket_is_refused() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .socket(benchmark.join("t2/environment/workspace/socket"))
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Err(BenchmarkError::OtherFiles {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn the_first_task_that_cannot_be_used_is_the_one_refused() {
    assert_eq!(
        refused(|directory, benchmark| {
            directory
                .remove_file(benchmark.join("t1/tests/test.sh"))
                .unwrap();
            directory
                .remove_file(benchmark.join("t2/instruction.md"))
                .unwrap();
        }),
        BenchmarkError::NoVerifier {
            task: "t1".to_owned()
        }
    );
}
