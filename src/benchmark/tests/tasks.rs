use crate::benchmark::{
    self, COMPOSE_FILES, DOCKERFILE, ENVIRONMENT, INSTRUCTION, TESTS, VERIFIER,
};
use crate::file_system::tests::double::Memory;

#[test]
fn the_tasks_of_a_benchmark_are_the_names_of_its_directories_in_order() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(
        &directory,
        directory.path(),
        &["rename-a-module", "add-a-flag", "fix-the-parser"],
    );
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec![
            "add-a-flag".to_owned(),
            "fix-the-parser".to_owned(),
            "rename-a-module".to_owned()
        ])
    );
}

#[test]
fn a_task_is_named_by_letters_digits_dots_underscores_and_hyphens() {
    let directory = Memory::default();
    let benchmark =
        test_inputs::benchmark_in(&directory, directory.path(), &["Task-1.a_b", "...", "2"]);
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec![
            "...".to_owned(),
            "2".to_owned(),
            "Task-1.a_b".to_owned()
        ])
    );
}

#[test]
fn a_file_in_a_benchmark_is_not_a_task() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .write(benchmark.join("README.md"), "Two tasks.\n")
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_directory_given_through_a_symbolic_link_is_a_task() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1"]);
    let elsewhere = test_inputs::task_in(&directory, directory.path(), "kept-elsewhere");
    directory.symlink(elsewhere, benchmark.join("t2")).unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_may_hold_more_than_its_instruction_its_tests_and_its_environment() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .create_dir(benchmark.join("t1/hidden-tests"))
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_that_holds_no_directory_named_workspace_is_a_task() {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .remove_dir_all(benchmark.join("t2/environment/workspace"))
        .unwrap();
    assert_eq!(
        benchmark::tasks(&directory, &benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn the_things_a_task_holds_are_named_instruction_md_tests_and_test_sh() {
    assert_eq!(
        [INSTRUCTION, TESTS, VERIFIER],
        ["instruction.md", "tests", "test.sh"]
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
