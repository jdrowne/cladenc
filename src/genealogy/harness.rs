use std::collections::BTreeSet;

use serde::Serialize;

use super::{AssessmentOutcome, HarnessId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessState {
    BeingCreated,
    Created,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Harness {
    id: HarnessId,
    parent: Option<HarnessId>,
    state: HarnessState,
    given: BTreeSet<String>,
    running: BTreeSet<String>,
    successes: u64,
    failures: u64,
    clade_successes: u64,
    clade_failures: u64,
}

impl Harness {
    pub(super) fn new(id: HarnessId, parent: Option<HarnessId>, state: HarnessState) -> Self {
        Self {
            id,
            parent,
            state,
            given: BTreeSet::new(),
            running: BTreeSet::new(),
            successes: 0,
            failures: 0,
            clade_successes: 0,
            clade_failures: 0,
        }
    }

    pub fn id(&self) -> HarnessId {
        self.id
    }

    pub fn parent(&self) -> Option<HarnessId> {
        self.parent
    }

    pub fn state(&self) -> HarnessState {
        self.state
    }

    pub fn successes(&self) -> u64 {
        self.successes
    }

    pub fn failures(&self) -> u64 {
        self.failures
    }

    pub fn clade_successes(&self) -> u64 {
        self.clade_successes
    }

    pub fn clade_failures(&self) -> u64 {
        self.clade_failures
    }

    pub fn clade_success_rate(&self) -> Option<f64> {
        let finished = self.clade_successes + self.clade_failures;
        (finished > 0).then(|| self.clade_successes as f64 / finished as f64)
    }

    pub fn tasks_not_given<'a>(&self, tasks: &'a [String]) -> Vec<&'a str> {
        tasks
            .iter()
            .map(String::as_str)
            .filter(|task| !self.given.contains(*task))
            .collect()
    }

    pub(super) fn set_state(&mut self, state: HarnessState) {
        self.state = state;
    }

    pub(super) fn give(&mut self, task: &str) -> bool {
        self.given.insert(task.to_owned()) && self.running.insert(task.to_owned())
    }

    pub(super) fn finish(&mut self, task: &str, outcome: AssessmentOutcome) -> bool {
        let was_running = self.running.remove(task);
        if was_running {
            match outcome {
                AssessmentOutcome::Success => self.successes += 1,
                AssessmentOutcome::Failure | AssessmentOutcome::TimedOut => self.failures += 1,
            }
        }
        was_running
    }

    pub(super) fn count_in_clade(&mut self, outcome: AssessmentOutcome) {
        match outcome {
            AssessmentOutcome::Success => self.clade_successes += 1,
            AssessmentOutcome::Failure | AssessmentOutcome::TimedOut => self.clade_failures += 1,
        }
    }
}
