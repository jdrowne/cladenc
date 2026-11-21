use std::fs;
use std::os::unix::fs::symlink;

use crate::benchmark::{
    self, COMPOSE_FILES, DOCKERFILE, ENVIRONMENT, GRADER, SPECIFICATION, TESTS, WORKSPACE,
};

#[test]
fn the_tasks_of_a_benchmark_are_the_names_of_its_directories_in_order() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(
        directory.path(),
        &["rename-a-module", "add-a-flag", "fix-the-parser"],
    );
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec![
            "add-a-flag".to_owned(),
            "fix-the-parser".to_owned(),
            "rename-a-module".to_owned()
        ])
    );
}

#[test]
fn a_task_is_named_by_letters_digits_dots_underscores_and_hyphens() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["Task-1.a_b", "...", "2"]);
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec![
            "...".to_owned(),
            "2".to_owned(),
            "Task-1.a_b".to_owned()
        ])
    );
}

#[test]
fn a_file_in_a_benchmark_is_not_a_task() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::write(benchmark.join("README.md"), "Two tasks.\n").unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_directory_given_through_a_symbolic_link_is_a_task() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    let elsewhere = test_inputs::task(directory.path(), "kept-elsewhere");
    symlink(elsewhere, benchmark.join("t2")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_may_hold_more_than_its_specification_its_workspace_and_its_tests() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::create_dir(benchmark.join("t1/hidden-tests")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_that_starts_from_no_file_is_a_task() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_file(benchmark.join("t2/workspace/work.txt")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn the_things_a_task_holds_are_named_instruction_md_workspace_tests_and_test_sh() {
    assert_eq!(
        [SPECIFICATION, WORKSPACE, TESTS, GRADER],
        ["instruction.md", "workspace", "tests", "test.sh"]
    );
}

#[test]
fn the_environment_of_a_task_is_named_environment_and_its_dockerfile_dockerfile() {
    assert_eq!([ENVIRONMENT, DOCKERFILE], ["environment", "Dockerfile"]);
}

#[test]
fn the_compose_files_that_are_refused_are_the_four_names_compose_reads() {
    assert_eq!(
        COMPOSE_FILES,
        [
            "compose.yaml",
            "compose.yml",
            "docker-compose.yaml",
            "docker-compose.yml"
        ]
    );
}
