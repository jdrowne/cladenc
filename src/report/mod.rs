mod experiments;
mod lead;
mod leader;

use std::array;
use std::path::Path;

use serde::Serialize;

use crate::genealogy::{Experiment, Genealogy, Harness, HarnessId, HarnessState};
use crate::holdout::TaskSet;
use crate::journal::{self, JournalError, Seconds, Settings};

use experiments::Experiments;
use lead::Lead;
use leader::Leader;

const COLUMNS: usize = 15;

const NOT_ENDED: &str = "the directed evolution has not ended: what follows is its state so far\n";

const HEADER: [&str; COLUMNS] = [
    "id",
    "progenitor",
    "experiment",
    "state",
    "selection completed",
    "selection not completed",
    "corroboration completed",
    "corroboration not completed",
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
    ended: bool,
    model: String,
    model_url: String,
    procreations_started: u64,
    max_procreations: u64,
    #[serde(flatten)]
    leader: Leader,
    pub selection_tasks_completed_by_any: u64,
    pub selection_tasks: u64,
    pub corroboration_tasks_completed_by_any: u64,
    pub corroboration_tasks: u64,
    pub other_tasks_completed_by_any: u64,
    pub other_tasks: u64,
    corroboration_lead: Option<Lead>,
    experiments: Experiments,
    harnesses: Vec<HarnessReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct HarnessReport {
    id: HarnessId,
    progenitor: Option<HarnessId>,
    experiment: Option<Experiment>,
    state: HarnessState,
    selection_completed: u64,
    selection_not_completed: u64,
    corroboration_completed: u64,
    corroboration_not_completed: u64,
    other_completed: u64,
    other_not_completed: u64,
    clade_completed: u64,
    clade_not_completed: u64,
    clade_score: Option<f64>,
    attempt_seconds: f64,
    procreation_seconds: Option<f64>,
}

pub fn read(outputs_directory: &Path) -> Result<Report, JournalError> {
    journal::read(outputs_directory).map(|state| {
        Report::of(
            &state.settings,
            &state.genealogy,
            &state.seconds,
            state.best_harness,
        )
    })
}

impl Report {
    fn of(
        settings: &Settings,
        genealogy: &Genealogy,
        seconds: &[Seconds],
        best_harness: Option<HarnessId>,
    ) -> Self {
        let counted = |set| {
            let tasks = settings.tasks_of(set);
            let completed_by_any = tasks
                .iter()
                .filter(|task| genealogy.completed_by_any(task))
                .count();
            (completed_by_any as u64, tasks.len() as u64)
        };
        let (selection_tasks_completed_by_any, selection_tasks) = counted(TaskSet::Selection);
        let (corroboration_tasks_completed_by_any, corroboration_tasks) =
            counted(TaskSet::Corroboration);
        let (other_tasks_completed_by_any, other_tasks) = counted(TaskSet::NonHoldout);
        let leader = best_harness.map_or_else(
            || Leader::HarnessAheadSoFar(settings.artificial_selection().best_harness(genealogy)),
            Leader::BestHarness,
        );
        let harnesses = genealogy.harnesses();
        let corroboration_lead = harnesses
            .iter()
            .find(|harness| harness.id() == leader.id())
            .filter(|_| leader.id() != 0)
            .map(|ahead| {
                Lead::of(
                    leader,
                    ahead,
                    &harnesses[0],
                    &settings.tasks_of(TaskSet::Corroboration),
                )
            });
        Self {
            ended: best_harness.is_some(),
            model: settings.model.clone(),
            model_url: settings.model_url.clone(),
            procreations_started: genealogy.procreations_started(),
            max_procreations: settings.max_procreations,
            leader,
            selection_tasks_completed_by_any,
            selection_tasks,
            corroboration_tasks_completed_by_any,
            corroboration_tasks,
            other_tasks_completed_by_any,
            other_tasks,
            corroboration_lead,
            experiments: Experiments::of(genealogy, &settings.tasks_of(TaskSet::Corroboration)),
            harnesses: harnesses
                .iter()
                .enumerate()
                .map(|(place, harness)| {
                    HarnessReport::of(harness, seconds.get(place).copied().unwrap_or_default())
                })
                .collect(),
        }
    }

    pub(crate) fn has_ended(&self) -> bool {
        self.ended
    }

    pub(crate) fn json(&self) -> String {
        serde_json::to_string(self).expect("a report can be written as JSON")
    }

    pub(crate) fn table(&self) -> String {
        let mut rows = vec![HEADER.map(str::to_owned)];
        rows.extend(self.harnesses.iter().map(HarnessReport::cells));
        let widths: [usize; COLUMNS] =
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
        let opening = if self.ended { "" } else { NOT_ENDED };
        let lead = self.corroboration_lead.as_ref().map_or_else(
            || {
                format!(
                    "on the corroboration tasks: {} is the primogenitor, so there are no two \
                     harnesses to compare",
                    self.leader.named()
                )
            },
            |lead| lead.line(self.leader),
        );
        format!(
            "{opening}model {} at {}\nstarted {} of {} {procreations}, {}\ncompleted by at least \
             one harness: selection tasks {} of {}, corroboration tasks {} of {}, other tasks {} \
             of {}\n{lead}\n{}\n\n{lines}",
            self.model,
            self.model_url,
            self.procreations_started,
            self.max_procreations,
            self.leader,
            self.selection_tasks_completed_by_any,
            self.selection_tasks,
            self.corroboration_tasks_completed_by_any,
            self.corroboration_tasks,
            self.other_tasks_completed_by_any,
            self.other_tasks,
            self.experiments
        )
    }
}

impl HarnessReport {
    fn of(harness: &Harness, seconds: Seconds) -> Self {
        Self {
            id: harness.id(),
            progenitor: harness.progenitor(),
            experiment: harness.experiment(),
            state: harness.state(),
            selection_completed: harness.selection_completed(),
            selection_not_completed: harness.selection_not_completed(),
            corroboration_completed: harness.corroboration_completed(),
            corroboration_not_completed: harness.corroboration_not_completed(),
            other_completed: harness.other_completed(),
            other_not_completed: harness.other_not_completed(),
            clade_completed: harness.clade_completed(),
            clade_not_completed: harness.clade_not_completed(),
            clade_score: harness.clade_score(),
            attempt_seconds: (seconds.attempts * 1000.0).round() / 1000.0,
            procreation_seconds: seconds.procreation,
        }
    }

    fn cells(&self) -> [String; COLUMNS] {
        let state = match self.state {
            HarnessState::BeingCreated => "being created",
            HarnessState::Created => "created",
            HarnessState::Unchanged => "unchanged",
            HarnessState::Failed => "failed",
        };
        [
            self.id.to_string(),
            shown(self.progenitor.map(|progenitor| progenitor.to_string())),
            shown(
                self.experiment
                    .map(|experiment| experiment.child.to_string()),
            ),
            state.to_owned(),
            self.selection_completed.to_string(),
            self.selection_not_completed.to_string(),
            self.corroboration_completed.to_string(),
            self.corroboration_not_completed.to_string(),
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
