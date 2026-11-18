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
    model: String,
    model_url: String,
    procreations_started: u64,
    max_procreations: u64,
    best_harness: HarnessId,
    pub holdout_tasks_completed_by_any: u64,
    pub holdout_tasks: u64,
    pub other_tasks_completed_by_any: u64,
    pub other_tasks: u64,
    harnesses: Vec<HarnessReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct HarnessReport {
    id: HarnessId,
    progenitor: Option<HarnessId>,
    state: HarnessState,
    holdout_completed: u64,
    holdout_not_completed: u64,
    other_completed: u64,
    other_not_completed: u64,
    clade_completed: u64,
    clade_not_completed: u64,
    clade_score: Option<f64>,
    attempt_seconds: f64,
    procreation_seconds: Option<f64>,
}

pub fn read(outputs_directory: &Path) -> Result<Report, JournalError> {
    journal::read(outputs_directory)
        .map(|state| Report::of(&state.settings, &state.genealogy, &state.seconds))
}

impl Report {
    fn of(settings: &Settings, genealogy: &Genealogy, seconds: &[Seconds]) -> Self {
        let (held_out, others): (Vec<String>, Vec<String>) = settings
            .tasks()
            .into_iter()
            .partition(|task| settings.holdout_tasks.contains(task));
        let completed_by_any = |tasks: &[String]| {
            tasks
                .iter()
                .filter(|task| genealogy.completed_by_any(task))
                .count() as u64
        };
        Self {
            model: settings.model.clone(),
            model_url: settings.model_url.clone(),
            procreations_started: genealogy.procreations_started(),
            max_procreations: settings.max_procreations,
            best_harness: settings.artificial_selection().best_harness(genealogy),
            holdout_tasks_completed_by_any: completed_by_any(&held_out),
            holdout_tasks: held_out.len() as u64,
            other_tasks_completed_by_any: completed_by_any(&others),
            other_tasks: others.len() as u64,
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

    pub(crate) fn json(&self) -> String {
        serde_json::to_string(self).expect("a report can be written as JSON")
    }

    pub(crate) fn table(&self) -> String {
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
            "model {} at {}\nstarted {} of {} {procreations}, best harness {}\ncompleted by at \
             least one harness: holdout tasks {} of {}, other tasks {} of {}\n\n{lines}",
            self.model,
            self.model_url,
            self.procreations_started,
            self.max_procreations,
            self.best_harness,
            self.holdout_tasks_completed_by_any,
            self.holdout_tasks,
            self.other_tasks_completed_by_any,
            self.other_tasks
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

#[cfg(test)]
mod tests;
