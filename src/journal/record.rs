use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JournalError;
use crate::artificial_selection::ArtificialSelection;
use crate::benchmark::Benchmark;
use crate::containers::Launch;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy, HarnessId};
use crate::holdout::TaskSet;
use crate::model::{self, Model};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    pub primogenitor: PathBuf,
    pub model: String,
    pub model_url: String,
    pub model_key_file: Option<PathBuf>,
    pub docker_image: String,
    pub dockerfile: Option<PathBuf>,
    pub docker_build_timeout: Option<u64>,
    pub environment_image_repository: String,
    pub harness_build_command: Option<String>,
    pub harness_run_command: String,
    pub benchmarks: Vec<Benchmark>,
    pub holdout_percentage: f64,
    pub holdout_tasks: Vec<String>,
    pub holdout_corroboration_percentage: f64,
    pub corroboration_tasks: Vec<String>,
    pub max_procreations: u64,
    pub max_concurrent_containers: u64,
    pub procreation_timeout: u64,
    pub attempt_timeout: u64,
    pub seed: u64,
}

impl Settings {
    pub(crate) fn model_key(&self) -> Result<Option<String>, PathBuf> {
        self.model_key_file
            .as_ref()
            .map(|file| model::read_key(file).ok_or_else(|| file.clone()))
            .transpose()
    }

    pub(super) fn model<'a>(&'a self, key: Option<&'a str>) -> Model<'a> {
        Model {
            name: &self.model,
            url: &self.model_url,
            key,
        }
    }

    pub(crate) fn launch<'a>(&'a self, key: Option<&'a str>) -> Launch<'a> {
        Launch {
            image: &self.docker_image,
            run_command: &self.harness_run_command,
            build_command: self.harness_build_command.as_deref(),
            model: self.model(key),
            procreation_timeout: self.procreation_timeout,
            attempt_timeout: self.attempt_timeout,
        }
    }

    pub(crate) fn genealogy(&self) -> Genealogy {
        let genealogy = match self.harness_build_command {
            Some(_) => Genealogy::unbuilt(),
            None => Genealogy::new(),
        };
        genealogy
            .selecting_by(&self.tasks_of(TaskSet::Selection))
            .corroborating_with(&self.tasks_of(TaskSet::Corroboration))
    }

    pub(crate) fn tasks(&self) -> Vec<String> {
        self.benchmarks
            .iter()
            .flat_map(|benchmark| benchmark.tasks.iter().map(|id| benchmark.task(id)))
            .collect()
    }

    pub(crate) fn environments(&self) -> Vec<(String, String)> {
        self.tasks()
            .into_iter()
            .enumerate()
            .map(|(index, task)| {
                (
                    task,
                    format!("{}:{index}", self.environment_image_repository),
                )
            })
            .collect()
    }

    pub(crate) fn tasks_of(&self, set: TaskSet) -> Vec<String> {
        self.tasks()
            .into_iter()
            .filter(|task| self.set_of(task) == set)
            .collect()
    }

    pub(crate) fn count_of(&self, benchmark: &Benchmark, set: TaskSet) -> u64 {
        benchmark
            .tasks
            .iter()
            .filter(|id| self.set_of(&benchmark.task(id)) == set)
            .count() as u64
    }

    fn set_of(&self, task: &str) -> TaskSet {
        let named_in = |tasks: &[String]| tasks.iter().any(|named| named == task);
        if !named_in(&self.holdout_tasks) {
            TaskSet::NonHoldout
        } else if named_in(&self.corroboration_tasks) {
            TaskSet::Corroboration
        } else {
            TaskSet::Selection
        }
    }

    pub(crate) fn artificial_selection(&self) -> ArtificialSelection {
        ArtificialSelection {
            max_procreations: self.max_procreations,
            tasks: self.tasks(),
            seed: self.seed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "record", rename_all = "snake_case")]
pub enum Record {
    Run {
        format: u32,
        #[serde(flatten)]
        settings: Box<Settings>,
    },
    PrimogenitorBuilt,
    ProcreationStarted {
        child: HarnessId,
        progenitor: HarnessId,
    },
    ProcreationFinished {
        child: HarnessId,
        outcome: CreationOutcome,
        seconds: f64,
    },
    BaselineStarted {
        harness: HarnessId,
        child: HarnessId,
        progenitor: HarnessId,
    },
    BaselineFinished {
        harness: HarnessId,
        outcome: CreationOutcome,
        seconds: f64,
    },
    AttemptStarted {
        harness: HarnessId,
        task: String,
    },
    AttemptFinished {
        harness: HarnessId,
        task: String,
        result: AttemptResult,
        seconds: f64,
    },
    DirectedEvolutionFinished {
        best_harness: HarnessId,
    },
}

impl Record {
    pub(crate) fn line(&self) -> Result<String, JournalError> {
        serde_json::to_string(self).map_err(|error| JournalError::Io(error.into()))
    }
}
