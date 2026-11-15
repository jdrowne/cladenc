use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JournalError;
use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};
use crate::job::Launch;
use crate::policy::Policy;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    pub progenitor: PathBuf,
    pub docker_image: String,
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
    pub fn launch(&self) -> Launch<'_> {
        Launch {
            image: &self.docker_image,
            run_command: &self.harness_run_command,
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
