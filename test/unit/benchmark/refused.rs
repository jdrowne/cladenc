use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::Path;

use cladenc::benchmark::{self, BenchmarkError};

fn refused(change: impl FnOnce(&Path)) -> BenchmarkError {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    change(&benchmark);
    benchmark::tasks(&benchmark).unwrap_err()
}

fn no_specification() -> BenchmarkError {
    BenchmarkError::NoSpecification {
        task: "t2".to_owned(),
    }
}

fn no_workspace() -> BenchmarkError {
    BenchmarkError::NoWorkspace {
        task: "t2".to_owned(),
    }
}

fn no_grader() -> BenchmarkError {
    BenchmarkError::NoGrader {
        task: "t2".to_owned(),
    }
}

#[test]
fn a_benchmark_that_does_not_exist_is_refused() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        benchmark::tasks(&directory.path().join("missing")),
        Err(BenchmarkError::NotADirectory)
    );
}

#[test]
fn a_benchmark_that_is_a_file_is_refused() {
    let directory = test_inputs::temporary_directory();
    let file = directory.path().join("benchmark");
    fs::write(&file, "t1\nt2\n").unwrap();
    assert_eq!(benchmark::tasks(&file), Err(BenchmarkError::NotADirectory));
}

#[test]
fn a_benchmark_that_holds_nothing_is_refused() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        benchmark::tasks(directory.path()),
        Err(BenchmarkError::NoTask)
    );
}

#[test]
fn a_benchmark_that_holds_files_and_no_directory_is_refused() {
    let directory = test_inputs::temporary_directory();
    fs::write(directory.path().join("specification"), "Fix the parser.\n").unwrap();
    assert_eq!(
        benchmark::tasks(directory.path()),
        Err(BenchmarkError::NoTask)
    );
}

#[test]
fn the_directory_of_one_task_is_refused_as_one_task_and_not_a_benchmark() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    assert_eq!(
        benchmark::tasks(&benchmark.join("t1")),
        Err(BenchmarkError::OneTask)
    );
}

#[test]
fn a_directory_with_a_space_in_its_name_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::create_dir(benchmark.join("task one")).unwrap()),
        BenchmarkError::NotAnId {
            directory: "task one".to_owned()
        }
    );
}

#[test]
fn a_task_with_no_specification_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_file(benchmark.join("t2/specification")).unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_a_directory_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/specification")).unwrap();
            fs::create_dir(benchmark.join("t2/specification")).unwrap();
        }),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/specification")).unwrap();
            symlink("../t1/specification", benchmark.join("t2/specification")).unwrap();
        }),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_says_nothing_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/specification"), " \n\n").unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_is_not_text_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/specification"), [0xff, 0xfe]).unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_whose_specification_holds_a_nul_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/specification"), "Fix\0it.\n").unwrap()),
        no_specification()
    );
}

#[test]
fn a_task_with_no_workspace_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_dir_all(benchmark.join("t2/workspace")).unwrap()),
        no_workspace()
    );
}

#[test]
fn a_task_whose_workspace_is_a_file_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_dir_all(benchmark.join("t2/workspace")).unwrap();
            fs::write(benchmark.join("t2/workspace"), "to do\n").unwrap();
        }),
        no_workspace()
    );
}

#[test]
fn a_task_with_no_grader_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_file(benchmark.join("t2/grader")).unwrap()),
        no_grader()
    );
}

#[test]
fn a_task_whose_grader_is_not_executable_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::set_permissions(
                benchmark.join("t2/grader"),
                fs::Permissions::from_mode(0o640),
            )
            .unwrap();
        }),
        no_grader()
    );
}

#[test]
fn a_task_whose_grader_is_a_directory_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/grader")).unwrap();
            fs::create_dir(benchmark.join("t2/grader")).unwrap();
        }),
        no_grader()
    );
}

#[test]
fn a_task_that_holds_a_socket_is_refused() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    let _socket = UnixListener::bind(benchmark.join("t2/workspace/socket")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::OtherFiles {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn the_first_task_that_cannot_be_used_is_the_one_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t1/grader")).unwrap();
            fs::remove_file(benchmark.join("t2/specification")).unwrap();
        }),
        BenchmarkError::NoGrader {
            task: "t1".to_owned()
        }
    );
}
