use std::array;
use std::path::Path;

use serde::Serialize;

use crate::artificial_selection::lower_bound;
use crate::genealogy::{Genealogy, Harness, HarnessId, HarnessState};
use crate::journal::{self, JournalError, Settings};

const HEADER: [&str; 9] = [
    "id",
    "progenitor",
    "state",
    "successes",
    "failures",
    "clade successes",
    "clade failures",
    "clade success rate",
    "lower bound",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub model: String,
    pub model_url: String,
    pub started: u64,
    pub budget: u64,
    pub best_harness: HarnessId,
    pub harnesses: Vec<HarnessReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HarnessReport {
    pub id: HarnessId,
    pub progenitor: Option<HarnessId>,
    pub state: HarnessState,
    pub successes: u64,
    pub failures: u64,
    pub clade_successes: u64,
    pub clade_failures: u64,
    pub clade_success_rate: Option<f64>,
    pub lower_bound: Option<f64>,
}

pub fn read(run_directory: &Path) -> Result<Report, JournalError> {
    journal::read(run_directory).map(|state| Report::of(&state.settings, &state.genealogy))
}

impl Report {
    pub fn of(settings: &Settings, genealogy: &Genealogy) -> Self {
        Self {
            model: settings.model.clone(),
            model_url: settings.model_url.clone(),
            started: genealogy.started(),
            budget: settings.budget,
            best_harness: settings.artificial_selection().best_harness(genealogy),
            harnesses: genealogy
                .harnesses()
                .iter()
                .map(|harness| HarnessReport::of(harness, settings.confidence))
                .collect(),
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("a report can be written as JSON")
    }

    pub fn table(&self) -> String {
        let mut rows = vec![HEADER.map(str::to_owned)];
        rows.extend(self.harnesses.iter().map(HarnessReport::cells));
        let widths: [usize; 9] =
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
        format!(
            "model {} at {}\nstarted {} of {}, best harness {}\n\n{lines}",
            self.model, self.model_url, self.started, self.budget, self.best_harness
        )
    }
}

impl HarnessReport {
    fn of(harness: &Harness, confidence: f64) -> Self {
        let created = harness.state() == HarnessState::Created;
        Self {
            id: harness.id(),
            progenitor: harness.progenitor(),
            state: harness.state(),
            successes: harness.successes(),
            failures: harness.failures(),
            clade_successes: harness.clade_successes(),
            clade_failures: harness.clade_failures(),
            clade_success_rate: harness.clade_success_rate(),
            lower_bound: created
                .then(|| lower_bound(harness.successes(), harness.failures(), confidence)),
        }
    }

    fn cells(&self) -> [String; 9] {
        let state = match self.state {
            HarnessState::BeingCreated => "being created",
            HarnessState::Created => "created",
            HarnessState::Failed => "failed",
        };
        [
            self.id.to_string(),
            shown(self.progenitor.map(|progenitor| progenitor.to_string())),
            state.to_owned(),
            self.successes.to_string(),
            self.failures.to_string(),
            self.clade_successes.to_string(),
            self.clade_failures.to_string(),
            shown(self.clade_success_rate.map(|rate| format!("{rate:.3}"))),
            shown(self.lower_bound.map(|bound| format!("{bound:.3}"))),
        ]
    }
}

fn shown(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_owned())
}
