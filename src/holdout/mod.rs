use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

use crate::benchmark::Benchmark;

const STREAM: u64 = u64::MAX;
const CORROBORATION_STREAM: u64 = u64::MAX - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaskSet {
    Selection,
    Corroboration,
    NonHoldout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HoldoutError {
    OneTask {
        benchmark: String,
    },
    NoHoldoutTask {
        benchmark: String,
        tasks: usize,
    },
    NoOtherTask {
        benchmark: String,
        tasks: usize,
    },
    OneHoldoutTask {
        benchmark: String,
    },
    NoCorroborationTask {
        benchmark: String,
        holdout_tasks: usize,
    },
    NoSelectionTask {
        benchmark: String,
        holdout_tasks: usize,
    },
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
            Self::OneHoldoutTask { benchmark } => write!(
                formatter,
                "the benchmark `{benchmark}` has one holdout task, which cannot be divided"
            ),
            Self::NoCorroborationTask {
                benchmark,
                holdout_tasks,
            } => write!(
                formatter,
                "the benchmark `{benchmark}` has {holdout_tasks} holdout tasks, and none of them \
                 would be a corroboration task"
            ),
            Self::NoSelectionTask {
                benchmark,
                holdout_tasks,
            } => write!(
                formatter,
                "the benchmark `{benchmark}` has {holdout_tasks} holdout tasks, and every one of \
                 them would be a corroboration task"
            ),
        }
    }
}

impl Error for HoldoutError {}

enum Undivided {
    One,
    NoPart,
    NoRest,
}

impl Undivided {
    fn held_out(self, benchmark: &Benchmark, tasks: usize) -> HoldoutError {
        let benchmark = benchmark.name();
        match self {
            Self::One => HoldoutError::OneTask { benchmark },
            Self::NoPart => HoldoutError::NoHoldoutTask { benchmark, tasks },
            Self::NoRest => HoldoutError::NoOtherTask { benchmark, tasks },
        }
    }

    fn corroborated(self, benchmark: &Benchmark, holdout_tasks: usize) -> HoldoutError {
        let benchmark = benchmark.name();
        match self {
            Self::One => HoldoutError::OneHoldoutTask { benchmark },
            Self::NoPart => HoldoutError::NoCorroborationTask {
                benchmark,
                holdout_tasks,
            },
            Self::NoRest => HoldoutError::NoSelectionTask {
                benchmark,
                holdout_tasks,
            },
        }
    }
}

fn count(tasks: usize, percentage: f64) -> usize {
    (tasks as f64 * percentage / 100.0).round() as usize
}

pub(crate) fn draw(
    benchmarks: &[Benchmark],
    percentage: f64,
    seed: u64,
) -> Result<Vec<String>, HoldoutError> {
    let mut generator = generator(seed, STREAM);
    let mut held_out = Vec::new();
    for benchmark in benchmarks {
        let tasks = tasks_of(benchmark);
        let amount = divided(tasks.len(), percentage)
            .map_err(|undivided| undivided.held_out(benchmark, tasks.len()))?;
        held_out.extend(drawn(&mut generator, &tasks, amount));
    }
    Ok(held_out)
}

pub(crate) fn draw_corroboration(
    benchmarks: &[Benchmark],
    holdout_tasks: &[String],
    percentage: f64,
    seed: u64,
) -> Result<Vec<String>, HoldoutError> {
    let mut generator = generator(seed, CORROBORATION_STREAM);
    let mut corroboration = Vec::new();
    for benchmark in benchmarks {
        let mut held_out = tasks_of(benchmark);
        held_out.retain(|task| holdout_tasks.contains(task));
        let amount = divided(held_out.len(), percentage)
            .map_err(|undivided| undivided.corroborated(benchmark, held_out.len()))?;
        corroboration.extend(drawn(&mut generator, &held_out, amount));
    }
    Ok(corroboration)
}

fn generator(seed: u64, stream: u64) -> ChaCha8Rng {
    let mut generator = ChaCha8Rng::seed_from_u64(seed);
    generator.set_stream(stream);
    generator
}

fn tasks_of(benchmark: &Benchmark) -> Vec<String> {
    benchmark
        .tasks
        .iter()
        .map(|id| benchmark.task(id))
        .collect()
}

fn drawn(generator: &mut ChaCha8Rng, tasks: &[String], amount: usize) -> Vec<String> {
    let mut places = index::sample(generator, tasks.len(), amount).into_vec();
    places.sort_unstable();
    places
        .into_iter()
        .map(|place| tasks[place].clone())
        .collect()
}

fn divided(whole: usize, percentage: f64) -> Result<usize, Undivided> {
    let part = count(whole, percentage);
    if whole == 1 {
        Err(Undivided::One)
    } else if part == 0 {
        Err(Undivided::NoPart)
    } else if part >= whole {
        Err(Undivided::NoRest)
    } else {
        Ok(part)
    }
}

#[cfg(test)]
mod tests;
