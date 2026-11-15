use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JournalError;
use crate::genealogy::{EvaluationOutcome, Genealogy, HarnessId, ProcreationOutcome};
use crate::job::Launch;
use crate::model::{self, Model};
use crate::policy::Policy;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    pub progenitor: PathBuf,
    pub model: String,
    pub model_url: String,
    pub model_key_file: Option<PathBuf>,
    pub docker_image: String,
    pub harness_build_command: Option<String>,
    pub harness_run_command: String,
    pub tasks: Vec<String>,
    pub evaluator: PathBuf,
    pub budget: u64,
    pub genealogy_growth: f64,
    pub confidence: f64,
    pub initial_procreations: u64,
    pub workers: u64,
    pub procreation_timeout: u64,
    pub evaluation_timeout: u64,
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
        }
    }

    pub fn genealogy(&self) -> Genealogy {
        match self.harness_build_command {
            Some(_) => Genealogy::unbuilt(),
            None => Genealogy::new(),
        }
    }

    pub fn policy(&self) -> Policy {
        Policy {
            budget: self.budget,
            growth: self.genealogy_growth,
            confidence: self.confidence,
            tasks: self.tasks.clone(),
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
        settings: Settings,
    },
    ProgenitorBuilt,
    ProcreationStarted {
        child: HarnessId,
        parent: HarnessId,
    },
    ProcreationFinished {
        child: HarnessId,
        outcome: ProcreationOutcome,
    },
    EvaluationStarted {
        harness: HarnessId,
        task: String,
    },
    EvaluationFinished {
        harness: HarnessId,
        task: String,
        outcome: EvaluationOutcome,
    },
    SearchFinished {
        best_harness: HarnessId,
    },
}

impl Record {
    pub fn line(&self) -> Result<String, JournalError> {
        serde_json::to_string(self).map_err(|error| JournalError::Io(error.into()))
    }
}
