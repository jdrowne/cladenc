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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Percentages {
    pub(crate) selection: f64,
    pub(crate) corroboration: f64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Drawn {
    pub(crate) holdout_tasks: Vec<String>,
    pub(crate) corroboration_tasks: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HoldoutError {
    TooFewTasks { benchmark: String },
    NoSelectionTask { benchmark: String, tasks: usize },
    NoCorroborationTask { benchmark: String, tasks: usize },
    NoNonHoldoutTask { benchmark: String, tasks: usize },
}

impl Display for HoldoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewTasks { benchmark } => write!(
                formatter,
                "the benchmark `{benchmark}` holds fewer than three tasks, which cannot be divided \
                 into a selection task, a corroboration task and a non-holdout task"
            ),
            Self::NoSelectionTask { benchmark, tasks } => write!(
                formatter,
                "the benchmark `{benchmark}` holds {tasks} tasks, and none of them would be a \
                 selection task"
            ),
            Self::NoCorroborationTask { benchmark, tasks } => write!(
                formatter,
                "the benchmark `{benchmark}` holds {tasks} tasks, and none of them would be a \
                 corroboration task"
            ),
            Self::NoNonHoldoutTask { benchmark, tasks } => write!(
                formatter,
                "the benchmark `{benchmark}` holds {tasks} tasks, and every one of them would be \
                 held out"
            ),
        }
    }
}

impl Error for HoldoutError {}

struct Counts {
    selection: usize,
    corroboration: usize,
}

fn count(tasks: usize, percentage: f64) -> usize {
    (tasks as f64 * percentage / 100.0).round() as usize
}

fn counts(
    benchmark: &Benchmark,
    tasks: usize,
    percentages: Percentages,
) -> Result<Counts, HoldoutError> {
    let selection = count(tasks, percentages.selection);
    let corroboration = count(tasks, percentages.corroboration);
    let name = benchmark.name();
    if tasks < 3 {
        Err(HoldoutError::TooFewTasks { benchmark: name })
    } else if selection == 0 {
        Err(HoldoutError::NoSelectionTask {
            benchmark: name,
            tasks,
        })
    } else if corroboration == 0 {
        Err(HoldoutError::NoCorroborationTask {
            benchmark: name,
            tasks,
        })
    } else if selection + corroboration >= tasks {
        Err(HoldoutError::NoNonHoldoutTask {
            benchmark: name,
            tasks,
        })
    } else {
        Ok(Counts {
            selection,
            corroboration,
        })
    }
}

pub(crate) fn draw(
    benchmarks: &[Benchmark],
    percentages: Percentages,
    seed: u64,
) -> Result<Drawn, HoldoutError> {
    let mut holdout_generator = generator(seed, STREAM);
    let mut corroboration_generator = generator(seed, CORROBORATION_STREAM);
    let mut drawn = Drawn::default();
    for benchmark in benchmarks {
        let tasks = tasks_of(benchmark);
        let counts = counts(benchmark, tasks.len(), percentages)?;
        let held_out = sampled(
            &mut holdout_generator,
            &tasks,
            counts.selection + counts.corroboration,
        );
        drawn.corroboration_tasks.extend(sampled(
            &mut corroboration_generator,
            &held_out,
            counts.corroboration,
        ));
        drawn.holdout_tasks.extend(held_out);
    }
    Ok(drawn)
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

fn sampled(generator: &mut ChaCha8Rng, tasks: &[String], amount: usize) -> Vec<String> {
    let mut places = index::sample(generator, tasks.len(), amount).into_vec();
    places.sort_unstable();
    places
        .into_iter()
        .map(|place| tasks[place].clone())
        .collect()
}

#[cfg(test)]
mod tests;
