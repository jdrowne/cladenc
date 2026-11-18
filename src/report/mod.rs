use std::array;
use std::path::Path;

use serde::Serialize;

use crate::genealogy::{Genealogy, Harness, HarnessId, HarnessState};
use crate::journal::{self, JournalError, Seconds, Settings};

const HEADER: [&str; 12] = [
    "id",
    "progenitor",
    "state",
    "holdout completed",
    "holdout not completed",
    "other completed",
    "other not completed",
    "clade completed",
    "clade not completed",
    "clade score",
    "attempt seconds",
    "procreation seconds",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub model: String,
    pub model_url: String,
    pub procreations_started: u64,
    pub max_procreations: u64,
    pub best_harness: HarnessId,
    pub harnesses: Vec<HarnessReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HarnessReport {
    pub id: HarnessId,
    pub progenitor: Option<HarnessId>,
    pub state: HarnessState,
    pub holdout_completed: u64,
    pub holdout_not_completed: u64,
    pub other_completed: u64,
    pub other_not_completed: u64,
    pub clade_completed: u64,
    pub clade_not_completed: u64,
    pub clade_score: Option<f64>,
    pub attempt_seconds: f64,
    pub procreation_seconds: Option<f64>,
}

pub fn read(outputs_directory: &Path) -> Result<Report, JournalError> {
    journal::read(outputs_directory)
        .map(|state| Report::of(&state.settings, &state.genealogy, &state.seconds))
}

impl Report {
    pub fn of(settings: &Settings, genealogy: &Genealogy, seconds: &[Seconds]) -> Self {
        Self {
            model: settings.model.clone(),
            model_url: settings.model_url.clone(),
            procreations_started: genealogy.procreations_started(),
            max_procreations: settings.max_procreations,
            best_harness: settings.artificial_selection().best_harness(genealogy),
            harnesses: genealogy
                .harnesses()
                .iter()
                .enumerate()
                .map(|(place, harness)| {
                    HarnessReport::of(harness, seconds.get(place).copied().unwrap_or_default())
                })
                .collect(),
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("a report can be written as JSON")
    }

    pub fn table(&self) -> String {
        let mut rows = vec![HEADER.map(str::to_owned)];
        rows.extend(self.harnesses.iter().map(HarnessReport::cells));
        let widths: [usize; 12] =
            array::from_fn(|column| rows.iter().map(|row| row[column].len()).fold(0, usize::max));
        let lines: String = rows
            .iter()
            .map(|row| {
                let line: String = row
                    .iter()
                    .zip(widths)
                    .map(|(cell, width)| format!("{cell:<width$}  "))
                    .collect();
                format!("{}\n", line.trim_end())
            })
            .collect();
        let procreations = if self.max_procreations == 1 {
            "procreation"
        } else {
            "procreations"
        };
        format!(
            "model {} at {}\nstarted {} of {} {procreations}, best harness {}\n\n{lines}",
            self.model,
            self.model_url,
            self.procreations_started,
            self.max_procreations,
            self.best_harness
        )
    }
}

impl HarnessReport {
    fn of(harness: &Harness, seconds: Seconds) -> Self {
        Self {
            id: harness.id(),
            progenitor: harness.progenitor(),
            state: harness.state(),
            holdout_completed: harness.holdout_completed(),
            holdout_not_completed: harness.holdout_not_completed(),
            other_completed: harness.other_completed(),
            other_not_completed: harness.other_not_completed(),
            clade_completed: harness.clade_completed(),
            clade_not_completed: harness.clade_not_completed(),
            clade_score: harness.clade_score(),
            attempt_seconds: (seconds.attempts * 1000.0).round() / 1000.0,
            procreation_seconds: seconds.procreation,
        }
    }

    fn cells(&self) -> [String; 12] {
        let state = match self.state {
            HarnessState::BeingCreated => "being created",
            HarnessState::Created => "created",
            HarnessState::Failed => "failed",
        };
        [
            self.id.to_string(),
            shown(self.progenitor.map(|progenitor| progenitor.to_string())),
            state.to_owned(),
            self.holdout_completed.to_string(),
            self.holdout_not_completed.to_string(),
            self.other_completed.to_string(),
            self.other_not_completed.to_string(),
            self.clade_completed.to_string(),
            self.clade_not_completed.to_string(),
            shown(self.clade_score.map(|rate| format!("{rate:.3}"))),
            format!("{:.1}", self.attempt_seconds),
            shown(
                self.procreation_seconds
                    .map(|seconds| format!("{seconds:.1}")),
            ),
        ]
    }
}

fn shown(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_owned())
}
