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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counts {
    passed: u64,
    failed: u64,
    inconclusive: u64,
}

impl Counts {
    fn count(&mut self, result: TrialResult) {
        match result {
            TrialResult::Pass => self.passed += 1,
            TrialResult::Fail => self.failed += 1,
            TrialResult::Inconclusive(_) => self.inconclusive += 1,
        }
    }

    fn finished(self) -> u64 {
        self.passed + self.failed + self.inconclusive
    }
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
    selection: Counts,
    corroboration: Counts,
    non_holdout: Counts,
    clade: Counts,
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
            selection: Counts::default(),
            corroboration: Counts::default(),
            non_holdout: Counts::default(),
            clade: Counts::default(),
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

    pub(crate) fn selection_passed(&self) -> u64 {
        self.selection.passed
    }

    pub(crate) fn selection_failed(&self) -> u64 {
        self.selection.failed
    }

    pub(crate) fn selection_inconclusive(&self) -> u64 {
        self.selection.inconclusive
    }

    pub(crate) fn corroboration_passed(&self) -> u64 {
        self.corroboration.passed
    }

    pub(crate) fn corroboration_failed(&self) -> u64 {
        self.corroboration.failed
    }

    pub(crate) fn corroboration_inconclusive(&self) -> u64 {
        self.corroboration.inconclusive
    }

    pub(crate) fn non_holdout_passed(&self) -> u64 {
        self.non_holdout.passed
    }

    pub(crate) fn non_holdout_failed(&self) -> u64 {
        self.non_holdout.failed
    }

    pub(crate) fn non_holdout_inconclusive(&self) -> u64 {
        self.non_holdout.inconclusive
    }

    pub(crate) fn clade_passed(&self) -> u64 {
        self.clade.passed
    }

    pub(crate) fn clade_failed(&self) -> u64 {
        self.clade.failed
    }

    pub(crate) fn clade_inconclusive(&self) -> u64 {
        self.clade.inconclusive
    }

    pub(crate) fn clade_failed_or_inconclusive(&self) -> u64 {
        self.clade.failed + self.clade.inconclusive
    }

    pub(crate) fn clade_completion_rate(&self) -> Option<f64> {
        let finished = self.clade.finished();
        (finished > 0).then(|| self.clade.passed as f64 / finished as f64)
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
            if result == TrialResult::Pass {
                self.completed.insert(task.to_owned());
            }
            let counts = match set {
                TaskSet::Selection => &mut self.selection,
                TaskSet::Corroboration => &mut self.corroboration,
                TaskSet::NonHoldout => &mut self.non_holdout,
            };
            counts.count(result);
        }
        was_running
    }

    pub(super) fn count_in_clade(&mut self, result: TrialResult) {
        self.clade.count(result);
    }
}
