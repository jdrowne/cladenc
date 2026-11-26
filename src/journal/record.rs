use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JournalError;
use crate::artificial_selection::ArtificialSelection;
use crate::dataset::Dataset;
use crate::file_system::FileSystem;
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};
use crate::holdout::TaskSet;
use crate::model::{self, Model};
use crate::sandboxes::Launch;

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
    pub datasets: Vec<Dataset>,
    pub selection_tasks_percentage: f64,
    pub holdout_tasks: Vec<String>,
    pub corroboration_tasks_percentage: f64,
    pub corroboration_tasks: Vec<String>,
    pub max_diversifications: u64,
    pub max_concurrent_sandboxes: u64,
    pub diversification_timeout: u64,
    pub max_verifier_health_check_retries: u64,
    pub max_task_health_check_retries: u64,
    pub seed: u64,
}

impl Settings {
    pub(crate) fn model_key(&self, files: &dyn FileSystem) -> Result<Option<String>, PathBuf> {
        self.model_key_file
            .as_ref()
            .map(|file| model::read_key(files, file).ok_or_else(|| file.clone()))
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
            diversification_timeout: self.diversification_timeout,
            max_verifier_health_check_retries: self.max_verifier_health_check_retries,
            max_task_health_check_retries: self.max_task_health_check_retries,
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
        self.datasets
            .iter()
            .flat_map(|dataset| dataset.tasks.iter().map(|id| dataset.task(id)))
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

    pub(crate) fn count_of(&self, dataset: &Dataset, set: TaskSet) -> u64 {
        dataset
            .tasks
            .iter()
            .filter(|id| self.set_of(&dataset.task(id)) == set)
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
            max_diversifications: self.max_diversifications,
            tasks: self.tasks(),
            seed: self.seed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "record", rename_all = "snake_case")]
pub(crate) enum Record {
    Run {
        format: u32,
        #[serde(flatten)]
        settings: Box<Settings>,
    },
    PrimogenitorBuilt,
    DiversificationStarted {
        child: HarnessId,
        progenitor: HarnessId,
    },
    DiversificationFinished {
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
    TrialStarted {
        harness: HarnessId,
        task: String,
    },
    TrialFinished {
        harness: HarnessId,
        task: String,
        #[serde(flatten)]
        result: TrialResult,
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
