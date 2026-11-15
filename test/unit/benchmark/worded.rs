use cladenc::benchmark::BenchmarkError;

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
fn a_task_with_no_statement_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoStatement { task: task() }.to_string(),
        "task fix-the-parser holds no file of text named statement"
    );
}

#[test]
fn a_task_with_no_files_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoFiles { task: task() }.to_string(),
        "task fix-the-parser holds no directory named files"
    );
}

#[test]
fn a_task_with_no_check_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::NoCheck { task: task() }.to_string(),
        "task fix-the-parser holds no executable file named check"
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
