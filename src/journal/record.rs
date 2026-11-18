use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JournalError;
use crate::artificial_selection::ArtificialSelection;
use crate::benchmark::Benchmark;
use crate::containers::Launch;
use crate::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};
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
    pub harness_build_command: Option<String>,
    pub harness_run_command: String,
    pub benchmarks: Vec<Benchmark>,
    pub holdout_percentage: f64,
    pub holdout_tasks: Vec<String>,
    pub max_procreations: u64,
    pub genealogy_growth: f64,
    pub max_concurrent_containers: u64,
    pub procreation_timeout: u64,
    pub attempt_timeout: u64,
    pub seed: u64,
}

impl Settings {
    pub fn model_key(&self) -> Result<Option<String>, PathBuf> {
        self.model_key_file
            .as_ref()
            .map(|file| model::read_key(file).ok_or_else(|| file.clone()))
            .transpose()
    }

    pub fn model<'a>(&'a self, key: Option<&'a str>) -> Model<'a> {
        Model {
            name: &self.model,
            url: &self.model_url,
            key,
        }
    }

    pub fn launch<'a>(&'a self, key: Option<&'a str>) -> Launch<'a> {
        Launch {
            image: &self.docker_image,
            run_command: &self.harness_run_command,
            build_command: self.harness_build_command.as_deref(),
            model: self.model(key),
            procreation_timeout: self.procreation_timeout,
            attempt_timeout: self.attempt_timeout,
        }
    }

    pub fn genealogy(&self) -> Genealogy {
        let genealogy = match self.harness_build_command {
            Some(_) => Genealogy::unbuilt(),
            None => Genealogy::new(),
        };
        genealogy.holding_out(&self.holdout_tasks)
    }

    pub fn tasks(&self) -> Vec<String> {
        self.benchmarks
            .iter()
            .flat_map(|benchmark| benchmark.tasks.iter().map(|id| benchmark.task(id)))
            .collect()
    }

    pub fn held_out_of(&self, benchmark: &Benchmark) -> u64 {
        benchmark
            .tasks
            .iter()
            .filter(|id| self.holdout_tasks.contains(&benchmark.task(id)))
            .count() as u64
    }

    pub fn artificial_selection(&self) -> ArtificialSelection {
        ArtificialSelection {
            max_procreations: self.max_procreations,
            growth: self.genealogy_growth,
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
        outcome: ProcreationOutcome,
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
    pub fn line(&self) -> Result<String, JournalError> {
        serde_json::to_string(self).map_err(|error| JournalError::Io(error.into()))
    }
}
