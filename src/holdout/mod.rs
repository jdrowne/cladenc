use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

use crate::benchmark::Benchmark;

const STREAM: u64 = u64::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HoldoutError {
    OneTask { benchmark: String },
    NoHoldoutTask { benchmark: String, tasks: usize },
    NoOtherTask { benchmark: String, tasks: usize },
}

impl Display for HoldoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneTask { benchmark } => write!(
                formatter,
                "the benchmark `{benchmark}` holds one task, which cannot be divided"
            ),
            Self::NoHoldoutTask { benchmark, tasks } => write!(
                formatter,
                "the benchmark `{benchmark}` holds {tasks} tasks, and none of them would be held \
                 out"
            ),
            Self::NoOtherTask { benchmark, tasks } => write!(
                formatter,
                "the benchmark `{benchmark}` holds {tasks} tasks, and every one of them would be \
                 held out"
            ),
        }
    }
}

impl Error for HoldoutError {}

pub fn count(tasks: usize, percentage: f64) -> usize {
    (tasks as f64 * percentage / 100.0).round() as usize
}

pub fn draw(
    benchmarks: &[Benchmark],
    percentage: f64,
    seed: u64,
) -> Result<Vec<String>, HoldoutError> {
    let mut generator = ChaCha8Rng::seed_from_u64(seed);
    generator.set_stream(STREAM);
    let mut held_out = Vec::new();
    for benchmark in benchmarks {
        let amount = divided(benchmark, percentage)?;
        let mut places = index::sample(&mut generator, benchmark.tasks.len(), amount).into_vec();
        places.sort_unstable();
        held_out.extend(
            places
                .into_iter()
                .map(|place| benchmark.task(&benchmark.tasks[place])),
        );
    }
    Ok(held_out)
}

fn divided(benchmark: &Benchmark, percentage: f64) -> Result<usize, HoldoutError> {
    let tasks = benchmark.tasks.len();
    let held_out = count(tasks, percentage);
    if tasks == 1 {
        Err(HoldoutError::OneTask {
            benchmark: benchmark.name(),
        })
    } else if held_out == 0 {
        Err(HoldoutError::NoHoldoutTask {
            benchmark: benchmark.name(),
            tasks,
        })
    } else if held_out >= tasks {
        Err(HoldoutError::NoOtherTask {
            benchmark: benchmark.name(),
            tasks,
        })
    } else {
        Ok(held_out)
    }
}

#[cfg(test)]
mod tests;
