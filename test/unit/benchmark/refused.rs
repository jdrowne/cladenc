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

fn no_statement() -> BenchmarkError {
    BenchmarkError::NoStatement {
        task: "t2".to_owned(),
    }
}

fn no_files() -> BenchmarkError {
    BenchmarkError::NoFiles {
        task: "t2".to_owned(),
    }
}

fn no_check() -> BenchmarkError {
    BenchmarkError::NoCheck {
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
    fs::write(directory.path().join("statement"), "Fix the parser.\n").unwrap();
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
fn a_task_with_no_statement_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_file(benchmark.join("t2/statement")).unwrap()),
        no_statement()
    );
}

#[test]
fn a_task_whose_statement_is_a_directory_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/statement")).unwrap();
            fs::create_dir(benchmark.join("t2/statement")).unwrap();
        }),
        no_statement()
    );
}

#[test]
fn a_task_whose_statement_is_a_symbolic_link_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/statement")).unwrap();
            symlink("../t1/statement", benchmark.join("t2/statement")).unwrap();
        }),
        no_statement()
    );
}

#[test]
fn a_task_whose_statement_says_nothing_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/statement"), " \n\n").unwrap()),
        no_statement()
    );
}

#[test]
fn a_task_whose_statement_is_not_text_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/statement"), [0xff, 0xfe]).unwrap()),
        no_statement()
    );
}

#[test]
fn a_task_whose_statement_holds_a_nul_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::write(benchmark.join("t2/statement"), "Fix\0it.\n").unwrap()),
        no_statement()
    );
}

#[test]
fn a_task_with_no_files_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_dir_all(benchmark.join("t2/files")).unwrap()),
        no_files()
    );
}

#[test]
fn a_task_whose_files_are_one_file_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_dir_all(benchmark.join("t2/files")).unwrap();
            fs::write(benchmark.join("t2/files"), "to do\n").unwrap();
        }),
        no_files()
    );
}

#[test]
fn a_task_with_no_check_is_refused() {
    assert_eq!(
        refused(|benchmark| fs::remove_file(benchmark.join("t2/check")).unwrap()),
        no_check()
    );
}

#[test]
fn a_task_whose_check_is_not_executable_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::set_permissions(
                benchmark.join("t2/check"),
                fs::Permissions::from_mode(0o640),
            )
            .unwrap();
        }),
        no_check()
    );
}

#[test]
fn a_task_whose_check_is_a_directory_is_refused() {
    assert_eq!(
        refused(|benchmark| {
            fs::remove_file(benchmark.join("t2/check")).unwrap();
            fs::create_dir(benchmark.join("t2/check")).unwrap();
        }),
        no_check()
    );
}

#[test]
fn a_task_that_holds_a_socket_is_refused() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    let _socket = UnixListener::bind(benchmark.join("t2/files/socket")).unwrap();
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
            fs::remove_file(benchmark.join("t1/check")).unwrap();
            fs::remove_file(benchmark.join("t2/statement")).unwrap();
        }),
        BenchmarkError::NoCheck {
            task: "t1".to_owned()
        }
    );
}
