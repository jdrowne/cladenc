use std::collections::BTreeSet;

use serde::Serialize;

use super::{AttemptResult, HarnessId};

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
    progenitor: Option<HarnessId>,
    state: HarnessState,
    given: BTreeSet<String>,
    running: BTreeSet<String>,
    completed: u64,
    not_completed: u64,
    clade_completed: u64,
    clade_not_completed: u64,
}

impl Harness {
    pub(super) fn new(id: HarnessId, progenitor: Option<HarnessId>, state: HarnessState) -> Self {
        Self {
            id,
            progenitor,
            state,
            given: BTreeSet::new(),
            running: BTreeSet::new(),
            completed: 0,
            not_completed: 0,
            clade_completed: 0,
            clade_not_completed: 0,
        }
    }

    pub fn id(&self) -> HarnessId {
        self.id
    }

    pub fn progenitor(&self) -> Option<HarnessId> {
        self.progenitor
    }

    pub fn state(&self) -> HarnessState {
        self.state
    }

    pub fn completed(&self) -> u64 {
        self.completed
    }

    pub fn not_completed(&self) -> u64 {
        self.not_completed
    }

    pub fn clade_completed(&self) -> u64 {
        self.clade_completed
    }

    pub fn clade_not_completed(&self) -> u64 {
        self.clade_not_completed
    }

    pub fn clade_score(&self) -> Option<f64> {
        let finished = self.clade_completed + self.clade_not_completed;
        (finished > 0).then(|| self.clade_completed as f64 / finished as f64)
    }

    pub fn tasks_not_given<'a>(&self, tasks: &'a [String]) -> Vec<&'a str> {
        tasks
            .iter()
            .map(String::as_str)
            .filter(|task| !self.given.contains(*task))
            .collect()
    }

    pub fn has_attempted_every(&self, tasks: &[String]) -> bool {
        tasks
            .iter()
            .all(|task| self.given.contains(task) && !self.running.contains(task))
    }

    pub(super) fn set_state(&mut self, state: HarnessState) {
        self.state = state;
    }

    pub(super) fn give(&mut self, task: &str) -> bool {
        self.given.insert(task.to_owned()) && self.running.insert(task.to_owned())
    }

    pub(super) fn finish(&mut self, task: &str, result: AttemptResult) -> bool {
        let was_running = self.running.remove(task);
        if was_running {
            match result {
                AttemptResult::Completed => self.completed += 1,
                AttemptResult::NotCompleted | AttemptResult::TimedOut => self.not_completed += 1,
            }
        }
        was_running
    }

    pub(super) fn count_in_clade(&mut self, result: AttemptResult) {
        match result {
            AttemptResult::Completed => self.clade_completed += 1,
            AttemptResult::NotCompleted | AttemptResult::TimedOut => self.clade_not_completed += 1,
        }
    }
}
