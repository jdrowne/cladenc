use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

use crate::dataset::Dataset;

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
    TooFewTasks { dataset: String },
    NoSelectionTask { dataset: String, tasks: usize },
    NoCorroborationTask { dataset: String, tasks: usize },
    NoNonHoldoutTask { dataset: String, tasks: usize },
}

impl Display for HoldoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewTasks { dataset } => write!(
                formatter,
                "the dataset `{dataset}` holds fewer than three tasks, which cannot be divided \
                 into a selection task, a corroboration task and a non-holdout task"
            ),
            Self::NoSelectionTask { dataset, tasks } => write!(
                formatter,
                "the dataset `{dataset}` holds {tasks} tasks, and none of them would be a \
                 selection task"
            ),
            Self::NoCorroborationTask { dataset, tasks } => write!(
                formatter,
                "the dataset `{dataset}` holds {tasks} tasks, and none of them would be a \
                 corroboration task"
            ),
            Self::NoNonHoldoutTask { dataset, tasks } => write!(
                formatter,
                "the dataset `{dataset}` holds {tasks} tasks, and every one of them would be \
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
    dataset: &Dataset,
    tasks: usize,
    percentages: Percentages,
) -> Result<Counts, HoldoutError> {
    let selection = count(tasks, percentages.selection);
    let corroboration = count(tasks, percentages.corroboration);
    let name = dataset.name();
    if tasks < 3 {
        Err(HoldoutError::TooFewTasks { dataset: name })
    } else if selection == 0 {
        Err(HoldoutError::NoSelectionTask {
            dataset: name,
            tasks,
        })
    } else if corroboration == 0 {
        Err(HoldoutError::NoCorroborationTask {
            dataset: name,
            tasks,
        })
    } else if selection + corroboration >= tasks {
        Err(HoldoutError::NoNonHoldoutTask {
            dataset: name,
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
    datasets: &[Dataset],
    percentages: Percentages,
    seed: u64,
) -> Result<Drawn, HoldoutError> {
    let mut holdout_generator = generator(seed, STREAM);
    let mut corroboration_generator = generator(seed, CORROBORATION_STREAM);
    let mut drawn = Drawn::default();
    for dataset in datasets {
        let tasks = tasks_of(dataset);
        let counts = counts(dataset, tasks.len(), percentages)?;
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

fn tasks_of(dataset: &Dataset) -> Vec<String> {
    dataset.tasks.iter().map(|id| dataset.task(id)).collect()
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
