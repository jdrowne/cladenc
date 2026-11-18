use std::path::PathBuf;

use crate::benchmark::{self, Benchmark, BenchmarkError};

fn benchmark(directory: &str) -> Benchmark {
    Benchmark {
        directory: PathBuf::from(directory),
        tasks: vec!["t1".to_owned()],
    }
}

#[test]
fn benchmarks_in_directories_of_different_names_are_told_apart() {
    assert_eq!(
        benchmark::told_apart(&[
            benchmark("/data/parsing"),
            benchmark("/data/refactoring"),
            benchmark("/more/typing")
        ]),
        Ok(())
    );
}

#[test]
fn one_benchmark_is_told_apart_from_no_other() {
    assert_eq!(benchmark::told_apart(&[benchmark("/data/parsing")]), Ok(()));
}

#[test]
fn a_benchmark_that_is_given_twice_is_not_told_apart_from_itself() {
    assert_eq!(
        benchmark::told_apart(&[
            benchmark("/data/parsing"),
            benchmark("/data/refactoring"),
            benchmark("/data/parsing")
        ]),
        Err(BenchmarkError::GivenTwice {
            directory: PathBuf::from("/data/parsing")
        })
    );
}

#[test]
fn two_benchmarks_in_directories_of_the_same_name_are_not_told_apart() {
    assert_eq!(
        benchmark::told_apart(&[benchmark("/data/parsing"), benchmark("/more/parsing")]),
        Err(BenchmarkError::SameName {
            name: "parsing".to_owned()
        })
    );
}
