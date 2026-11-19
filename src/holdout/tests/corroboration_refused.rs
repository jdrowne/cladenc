use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::holdout::{self, HoldoutError};

fn benchmark(name: &str, tasks: usize) -> Benchmark {
    Benchmark {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task}")).collect(),
    }
}

fn holding_out(benchmark: &Benchmark, tasks: usize) -> Vec<String> {
    benchmark
        .tasks
        .iter()
        .take(tasks)
        .map(|id| benchmark.task(id))
        .collect()
}

fn refusal(held_out: usize, percentage: f64) -> HoldoutError {
    let bugs = benchmark("bugs", 10);
    let holdout_tasks = holding_out(&bugs, held_out);
    holdout::draw_corroboration(&[bugs], &holdout_tasks, percentage, 7).unwrap_err()
}

#[test]
fn a_benchmark_with_one_holdout_task_is_refused_as_one_that_cannot_be_divided() {
    assert_eq!(
        refusal(1, 50.0),
        HoldoutError::OneHoldoutTask {
            benchmark: "bugs".to_owned()
        }
    );
}

#[test]
fn a_benchmark_with_one_holdout_task_is_refused_whatever_the_percentage() {
    let bugs = benchmark("bugs", 10);
    let holdout_tasks = holding_out(&bugs, 1);
    assert!((1..100_u32).all(|percentage| {
        holdout::draw_corroboration(
            std::slice::from_ref(&bugs),
            &holdout_tasks,
            f64::from(percentage),
            7,
        )
        .is_err()
    }));
}

#[test]
fn a_percentage_that_makes_no_holdout_task_a_corroboration_task_is_refused() {
    assert_eq!(
        refusal(3, 10.0),
        HoldoutError::NoCorroborationTask {
            benchmark: "bugs".to_owned(),
            holdout_tasks: 3
        }
    );
}

#[test]
fn a_percentage_that_makes_every_holdout_task_a_corroboration_task_is_refused() {
    assert_eq!(
        refusal(3, 90.0),
        HoldoutError::NoSelectionTask {
            benchmark: "bugs".to_owned(),
            holdout_tasks: 3
        }
    );
}

#[test]
fn a_benchmark_with_no_holdout_task_is_refused_as_one_with_no_corroboration_task() {
    assert_eq!(
        refusal(0, 50.0),
        HoldoutError::NoCorroborationTask {
            benchmark: "bugs".to_owned(),
            holdout_tasks: 0
        }
    );
}

#[test]
fn among_several_benchmarks_the_one_whose_holdout_tasks_cannot_be_divided_is_named() {
    let bugs = benchmark("bugs", 10);
    let features = benchmark("features", 10);
    let holdout_tasks = [holding_out(&bugs, 6), holding_out(&features, 3)].concat();
    assert_eq!(
        holdout::draw_corroboration(&[bugs, features], &holdout_tasks, 10.0, 7).unwrap_err(),
        HoldoutError::NoCorroborationTask {
            benchmark: "features".to_owned(),
            holdout_tasks: 3
        }
    );
}
