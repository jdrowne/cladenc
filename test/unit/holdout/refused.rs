use std::path::PathBuf;

use cladenc::benchmark::Benchmark;
use cladenc::holdout::{self, HoldoutError};

fn benchmark(name: &str, tasks: usize) -> Benchmark {
    Benchmark {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task}")).collect(),
    }
}

fn refusal(benchmarks: &[Benchmark], percentage: f64) -> HoldoutError {
    holdout::draw(benchmarks, percentage, 7).unwrap_err()
}

#[test]
fn a_benchmark_of_one_task_is_refused_as_one_that_cannot_be_divided() {
    assert_eq!(
        refusal(&[benchmark("bugs", 1)], 50.0),
        HoldoutError::OneTask {
            benchmark: "bugs".to_owned()
        }
    );
}

#[test]
fn a_benchmark_of_one_task_is_refused_whatever_the_percentage() {
    assert!((1..100_u32).all(|percentage| {
        holdout::draw(&[benchmark("bugs", 1)], f64::from(percentage), 7).is_err()
    }));
}

#[test]
fn a_percentage_that_holds_out_no_task_of_a_benchmark_is_refused() {
    assert_eq!(
        refusal(&[benchmark("bugs", 3)], 10.0),
        HoldoutError::NoHoldoutTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn a_percentage_that_holds_out_every_task_of_a_benchmark_is_refused() {
    assert_eq!(
        refusal(&[benchmark("bugs", 3)], 90.0),
        HoldoutError::NoOtherTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn among_several_benchmarks_the_one_that_is_left_a_part_with_no_task_is_named() {
    assert_eq!(
        refusal(&[benchmark("bugs", 10), benchmark("features", 3)], 10.0),
        HoldoutError::NoHoldoutTask {
            benchmark: "features".to_owned(),
            tasks: 3
        }
    );
}
