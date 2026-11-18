use std::collections::BTreeSet;

use serde::Serialize;

use super::{AttemptResult, HarnessId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HarnessState {
    BeingCreated,
    Created,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Harness {
    id: HarnessId,
    progenitor: Option<HarnessId>,
    state: HarnessState,
    given: BTreeSet<String>,
    running: BTreeSet<String>,
    completed: BTreeSet<String>,
    holdout_completed: u64,
    holdout_not_completed: u64,
    other_completed: u64,
    other_not_completed: u64,
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
            completed: BTreeSet::new(),
            holdout_completed: 0,
            holdout_not_completed: 0,
            other_completed: 0,
            other_not_completed: 0,
            clade_completed: 0,
            clade_not_completed: 0,
        }
    }

    pub(crate) fn id(&self) -> HarnessId {
        self.id
    }

    pub(crate) fn progenitor(&self) -> Option<HarnessId> {
        self.progenitor
    }

    pub(crate) fn state(&self) -> HarnessState {
        self.state
    }

    pub(crate) fn holdout_completed(&self) -> u64 {
        self.holdout_completed
    }

    pub(crate) fn holdout_not_completed(&self) -> u64 {
        self.holdout_not_completed
    }

    pub(crate) fn other_completed(&self) -> u64 {
        self.other_completed
    }

    pub(crate) fn other_not_completed(&self) -> u64 {
        self.other_not_completed
    }

    pub(crate) fn clade_completed(&self) -> u64 {
        self.clade_completed
    }

    pub(crate) fn clade_not_completed(&self) -> u64 {
        self.clade_not_completed
    }

    pub(crate) fn clade_score(&self) -> Option<f64> {
        let finished = self.clade_completed + self.clade_not_completed;
        (finished > 0).then(|| self.clade_completed as f64 / finished as f64)
    }

    pub(crate) fn tasks_not_given<'a>(&self, tasks: &'a [String]) -> Vec<&'a str> {
        tasks
            .iter()
            .map(String::as_str)
            .filter(|task| !self.given.contains(*task))
            .collect()
    }

    pub(super) fn has_completed(&self, task: &str) -> bool {
        self.completed.contains(task)
    }

    pub(crate) fn has_attempted_every(&self, tasks: &[String]) -> bool {
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

    pub(super) fn finish(&mut self, task: &str, result: AttemptResult, held_out: bool) -> bool {
        let was_running = self.running.remove(task);
        if was_running {
            if result == AttemptResult::Completed {
                self.completed.insert(task.to_owned());
            }
            let count = match (held_out, result) {
                (true, AttemptResult::Completed) => &mut self.holdout_completed,
                (true, AttemptResult::NotCompleted | AttemptResult::TimedOut) => {
                    &mut self.holdout_not_completed
                }
                (false, AttemptResult::Completed) => &mut self.other_completed,
                (false, AttemptResult::NotCompleted | AttemptResult::TimedOut) => {
                    &mut self.other_not_completed
                }
            };
            *count += 1;
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
