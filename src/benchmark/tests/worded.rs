use std::path::PathBuf;

use crate::benchmark::BenchmarkError;

fn task() -> String {
    "fix-the-parser".to_owned()
}

#[test]
fn what_is_not_a_directory_is_said_to_be_none() {
    assert_eq!(
        BenchmarkError::NotADirectory.to_string(),
        "what was given is not a directory that can be read"
    );
}

#[test]
fn a_benchmark_with_no_directory_is_said_to_hold_no_task() {
    assert_eq!(
        BenchmarkError::NoTask.to_string(),
        "the directory given holds no directory, and so no task"
    );
}

#[test]
fn the_directory_of_one_task_is_said_to_be_one_task() {
    assert_eq!(
        BenchmarkError::OneTask.to_string(),
        "what was given is the directory of one task, and a benchmark is the directory that \
         holds the tasks"
    );
}

#[test]
fn a_directory_that_is_not_named_as_a_task_is_said_with_its_name() {
    assert_eq!(
        BenchmarkError::NotAnId {
            directory: "task one".to_owned()
        }
        .to_string(),
        "the directory `task one` in it is not named with letters, digits, dots, underscores and \
         hyphens alone, as a task is"
    );
}

#[test]
fn a_task_with_no_specification_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoSpecification { task: task() }.to_string(),
        "task fix-the-parser holds no file of text named specification"
    );
}

#[test]
fn a_task_with_no_workspace_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoWorkspace { task: task() }.to_string(),
        "task fix-the-parser holds no directory named workspace"
    );
}

#[test]
fn a_task_with_no_grader_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoGrader { task: task() }.to_string(),
        "task fix-the-parser holds no executable file named grader"
    );
}

#[test]
fn a_task_that_cannot_be_copied_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::OtherFiles { task: task() }.to_string(),
        "task fix-the-parser holds something that is not a regular file, a directory or a \
         symbolic link"
    );
}

#[test]
fn a_benchmark_that_is_not_named_as_one_is_said_with_the_name_of_its_directory() {
    assert_eq!(
        BenchmarkError::NotNamed {
            directory: "my benchmark".to_owned()
        }
        .to_string(),
        "its directory is named `my benchmark`, and the name of a benchmark is made of letters, \
         digits, dots, underscores and hyphens alone"
    );
}

#[test]
fn a_benchmark_that_is_given_twice_is_said_with_its_directory() {
    assert_eq!(
        BenchmarkError::GivenTwice {
            directory: PathBuf::from("/data/parsing")
        }
        .to_string(),
        "the benchmark /data/parsing is given twice"
    );
}

#[test]
fn two_benchmarks_of_the_same_name_are_said_with_the_name_and_why_it_matters() {
    assert_eq!(
        BenchmarkError::SameName {
            name: "parsing".to_owned()
        }
        .to_string(),
        "two of the benchmarks given are in directories named `parsing`, and a task is known by \
         the name of its benchmark and its id together"
    );
}
