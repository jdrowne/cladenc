use std::collections::BTreeSet;

use serde::Serialize;

use super::{Experiment, HarnessId, TrialResult};
use crate::holdout::TaskSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HarnessState {
    BeingCreated,
    Created,
    Unchanged,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Harness {
    id: HarnessId,
    progenitor: Option<HarnessId>,
    experiment: Option<Experiment>,
    state: HarnessState,
    given: BTreeSet<String>,
    running: BTreeSet<String>,
    completed: BTreeSet<String>,
    selection_completed: u64,
    selection_not_completed: u64,
    corroboration_completed: u64,
    corroboration_not_completed: u64,
    non_holdout_completed: u64,
    non_holdout_not_completed: u64,
    clade_completed: u64,
    clade_not_completed: u64,
}

impl Harness {
    pub(super) fn new(id: HarnessId, progenitor: Option<HarnessId>, state: HarnessState) -> Self {
        Self {
            id,
            progenitor,
            experiment: None,
            state,
            given: BTreeSet::new(),
            running: BTreeSet::new(),
            completed: BTreeSet::new(),
            selection_completed: 0,
            selection_not_completed: 0,
            corroboration_completed: 0,
            corroboration_not_completed: 0,
            non_holdout_completed: 0,
            non_holdout_not_completed: 0,
            clade_completed: 0,
            clade_not_completed: 0,
        }
    }

    pub(super) fn singleton_control_group(id: HarnessId, experiment: Experiment) -> Self {
        Self {
            experiment: Some(experiment),
            ..Self::new(id, None, HarnessState::BeingCreated)
        }
    }

    pub(crate) fn id(&self) -> HarnessId {
        self.id
    }

    pub(crate) fn progenitor(&self) -> Option<HarnessId> {
        self.progenitor
    }

    pub(crate) fn experiment(&self) -> Option<Experiment> {
        self.experiment
    }

    pub(crate) fn is_a_singleton_control_group_harness(&self) -> bool {
        self.experiment.is_some()
    }

    pub(crate) fn state(&self) -> HarnessState {
        self.state
    }

    pub(crate) fn selection_completed(&self) -> u64 {
        self.selection_completed
    }

    pub(crate) fn selection_not_completed(&self) -> u64 {
        self.selection_not_completed
    }

    pub(crate) fn corroboration_completed(&self) -> u64 {
        self.corroboration_completed
    }

    pub(crate) fn corroboration_not_completed(&self) -> u64 {
        self.corroboration_not_completed
    }

    pub(crate) fn non_holdout_completed(&self) -> u64 {
        self.non_holdout_completed
    }

    pub(crate) fn non_holdout_not_completed(&self) -> u64 {
        self.non_holdout_not_completed
    }

    pub(crate) fn clade_completed(&self) -> u64 {
        self.clade_completed
    }

    pub(crate) fn clade_not_completed(&self) -> u64 {
        self.clade_not_completed
    }

    pub(crate) fn clade_completion_rate(&self) -> Option<f64> {
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

    pub(crate) fn has_completed(&self, task: &str) -> bool {
        self.completed.contains(task)
    }

    pub(crate) fn has_trialled(&self, task: &str) -> bool {
        self.given.contains(task) && !self.running.contains(task)
    }

    pub(crate) fn has_trialled_every(&self, tasks: &[String]) -> bool {
        tasks.iter().all(|task| self.has_trialled(task))
    }

    pub(super) fn set_state(&mut self, state: HarnessState) {
        self.state = state;
    }

    pub(super) fn give(&mut self, task: &str) -> bool {
        self.given.insert(task.to_owned()) && self.running.insert(task.to_owned())
    }

    pub(super) fn finish(&mut self, task: &str, result: TrialResult, set: TaskSet) -> bool {
        let was_running = self.running.remove(task);
        if was_running {
            let completed = result == TrialResult::Completed;
            if completed {
                self.completed.insert(task.to_owned());
            }
            let count = match (set, completed) {
                (TaskSet::Selection, true) => &mut self.selection_completed,
                (TaskSet::Selection, false) => &mut self.selection_not_completed,
                (TaskSet::Corroboration, true) => &mut self.corroboration_completed,
                (TaskSet::Corroboration, false) => &mut self.corroboration_not_completed,
                (TaskSet::NonHoldout, true) => &mut self.non_holdout_completed,
                (TaskSet::NonHoldout, false) => &mut self.non_holdout_not_completed,
            };
            *count += 1;
        }
        was_running
    }

    pub(super) fn count_in_clade(&mut self, result: TrialResult) {
        match result {
            TrialResult::Completed => self.clade_completed += 1,
            TrialResult::NotCompleted | TrialResult::TimedOut => self.clade_not_completed += 1,
        }
    }
}
