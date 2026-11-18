use std::path::PathBuf;

use crate::benchmark::{Benchmark, BenchmarkError};

fn parsing() -> Benchmark {
    Benchmark {
        directory: PathBuf::from("/data/parsing"),
        tasks: vec!["fix-the-parser".to_owned()],
    }
}

fn read_in_a_directory_named(name: &str) -> Result<String, BenchmarkError> {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark_named(directory.path(), name, &["t1"]);
    Benchmark::read(&benchmark).map(|benchmark| benchmark.name())
}

#[test]
fn reading_a_benchmark_gives_its_directory_and_the_ids_of_its_tasks_in_order() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t2", "t1"]);
    assert_eq!(
        Benchmark::read(&benchmark),
        Ok(Benchmark {
            directory: benchmark,
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        })
    );
}

#[test]
fn the_name_of_a_benchmark_is_the_name_of_its_directory() {
    assert_eq!(parsing().name(), "parsing");
}

#[test]
fn a_task_is_known_by_the_name_of_its_benchmark_and_its_id_together() {
    assert_eq!(parsing().task("fix-the-parser"), "parsing/fix-the-parser");
}

#[test]
fn a_benchmark_is_named_by_letters_digits_dots_underscores_and_hyphens() {
    assert_eq!(
        read_in_a_directory_named("Bench-1.a_b"),
        Ok("Bench-1.a_b".to_owned())
    );
}

#[test]
fn a_benchmark_in_a_directory_with_a_space_in_its_name_is_refused() {
    assert_eq!(
        read_in_a_directory_named("my benchmark"),
        Err(BenchmarkError::NotNamed {
            directory: "my benchmark".to_owned()
        })
    );
}

#[test]
fn a_benchmark_given_by_a_path_that_ends_in_no_name_is_refused() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    assert_eq!(
        Benchmark::read(&benchmark.join("t1/..")),
        Err(BenchmarkError::NotNamed {
            directory: String::new()
        })
    );
}

#[test]
fn what_is_not_a_benchmark_is_refused_for_that_and_not_for_its_name() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        Benchmark::read(&directory.path().join("my benchmark")),
        Err(BenchmarkError::NotADirectory)
    );
}
