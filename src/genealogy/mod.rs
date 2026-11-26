mod error;
mod harness;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub use error::GenealogyError;
pub(crate) use harness::{Harness, HarnessState};

use crate::holdout::TaskSet;

pub(crate) type HarnessId = u64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreationOutcome {
    Created,
    Unchanged,
    Failed,
    BuildFailed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "result", content = "cause", rename_all = "snake_case")]
pub enum TrialResult {
    Pass,
    Fail,
    Inconclusive(Cause),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    NoVerdict,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Experiment {
    pub child: HarnessId,
    pub progenitor: HarnessId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Genealogy {
    harnesses: Vec<Harness>,
    started: u64,
    started_by_singleton_control_group_harnesses: u64,
    selection_tasks: BTreeSet<String>,
    corroboration_tasks: BTreeSet<String>,
}

impl Default for Genealogy {
    fn default() -> Self {
        Self::new()
    }
}

impl Genealogy {
    pub(crate) fn new() -> Self {
        Self::of_a_primogenitor(HarnessState::Created)
    }

    pub(crate) fn unbuilt() -> Self {
        Self::of_a_primogenitor(HarnessState::BeingCreated)
    }

    fn of_a_primogenitor(state: HarnessState) -> Self {
        Self {
            harnesses: vec![Harness::new(0, None, state)],
            started: 0,
            started_by_singleton_control_group_harnesses: 0,
            selection_tasks: BTreeSet::new(),
            corroboration_tasks: BTreeSet::new(),
        }
    }

    pub(crate) fn selecting_by(self, tasks: &[String]) -> Self {
        Self {
            selection_tasks: tasks.iter().cloned().collect(),
            ..self
        }
    }

    pub(crate) fn corroborating_with(self, tasks: &[String]) -> Self {
        Self {
            corroboration_tasks: tasks.iter().cloned().collect(),
            ..self
        }
    }

    pub(crate) fn awaits_build(&self) -> bool {
        self.harnesses[0].state() == HarnessState::BeingCreated
    }

    pub(crate) fn build_primogenitor(&mut self) -> Result<(), GenealogyError> {
        self.harness_in_state(0, HarnessState::BeingCreated)
            .ok_or(GenealogyError::PrimogenitorNotAwaitingBuild)?
            .set_state(HarnessState::Created);
        Ok(())
    }

    pub(crate) fn harnesses(&self) -> &[Harness] {
        &self.harnesses
    }

    pub(crate) fn harness_count(&self) -> u64 {
        self.harnesses.len() as u64
    }

    pub(crate) fn started(&self) -> u64 {
        self.started
    }

    pub(crate) fn diversifications_started(&self) -> u64 {
        self.harnesses
            .iter()
            .filter(|harness| harness.progenitor().is_some())
            .count() as u64
    }

    pub(crate) fn actions_started(&self) -> u64 {
        self.diversifications_started() + self.started
            - self.started_by_singleton_control_group_harnesses
    }

    pub(crate) fn completed_by_any(&self, task: &str) -> bool {
        self.harnesses.iter().any(|harness| {
            !harness.is_a_singleton_control_group_harness() && harness.has_completed(task)
        })
    }

    pub(crate) fn created_harnesses(&self) -> Vec<&Harness> {
        self.harnesses
            .iter()
            .filter(|harness| harness.state() == HarnessState::Created)
            .collect()
    }

    pub(crate) fn start_diversification(
        &mut self,
        progenitor: HarnessId,
    ) -> Result<HarnessId, GenealogyError> {
        if self
            .harness_in_state(progenitor, HarnessState::Created)
            .is_none()
        {
            return Err(GenealogyError::ProgenitorNotCreated { progenitor });
        }
        let child = self.harness_count();
        self.harnesses.push(Harness::new(
            child,
            Some(progenitor),
            HarnessState::BeingCreated,
        ));
        Ok(child)
    }

    pub(crate) fn end_diversification(
        &mut self,
        child: HarnessId,
        outcome: CreationOutcome,
    ) -> Result<(), GenealogyError> {
        self.being_created(child, false)
            .ok_or(GenealogyError::ChildNotBeingCreated { child })?
            .set_state(state_after(outcome));
        Ok(())
    }

    pub(crate) fn experiment_awaiting_its_baseline(&self) -> Option<Experiment> {
        self.harnesses.last().and_then(|child| {
            child
                .progenitor()
                .filter(|progenitor| *progenitor != 0)
                .map(|progenitor| Experiment {
                    child: child.id(),
                    progenitor,
                })
        })
    }

    pub(crate) fn start_baseline(
        &mut self,
        experiment: Experiment,
    ) -> Result<HarnessId, GenealogyError> {
        if self.experiment_awaiting_its_baseline() != Some(experiment) {
            return Err(GenealogyError::ExperimentNotAwaitingBaseline {
                child: experiment.child,
                progenitor: experiment.progenitor,
            });
        }
        let harness = self.harness_count();
        self.harnesses
            .push(Harness::singleton_control_group(harness, experiment));
        Ok(harness)
    }

    pub(crate) fn end_baseline(
        &mut self,
        harness: HarnessId,
        outcome: CreationOutcome,
    ) -> Result<(), GenealogyError> {
        self.being_created(harness, true)
            .ok_or(GenealogyError::SingletonControlGroupHarnessNotBeingCreated { harness })?
            .set_state(state_after(outcome));
        Ok(())
    }

    pub(crate) fn start_trial(
        &mut self,
        harness: HarnessId,
        task: &str,
    ) -> Result<(), GenealogyError> {
        let trialling = self
            .harness_in_state(harness, HarnessState::Created)
            .ok_or(GenealogyError::HarnessNotCreated { harness })?;
        if !trialling.give(task) {
            return Err(GenealogyError::TaskAlreadyGiven {
                harness,
                task: task.to_owned(),
            });
        }
        let in_an_experiment = trialling.is_a_singleton_control_group_harness();
        self.started += 1;
        self.started_by_singleton_control_group_harnesses += u64::from(in_an_experiment);
        Ok(())
    }

    pub(crate) fn end_trial(
        &mut self,
        harness: HarnessId,
        task: &str,
        result: TrialResult,
    ) -> Result<(), GenealogyError> {
        let set = self.set_of(task);
        let finished = self
            .harness_mut(harness)
            .is_some_and(|trialling| trialling.finish(task, result, set));
        if !finished {
            return Err(GenealogyError::TrialNotRunning {
                harness,
                task: task.to_owned(),
            });
        }
        let mut member = (set == TaskSet::Selection).then_some(harness);
        while let Some(ancestor) = member
            .and_then(|id| self.harness_mut(id))
            .filter(|ancestor| !ancestor.is_a_singleton_control_group_harness())
        {
            ancestor.count_in_clade(result);
            member = ancestor.progenitor();
        }
        Ok(())
    }

    fn set_of(&self, task: &str) -> TaskSet {
        if self.selection_tasks.contains(task) {
            TaskSet::Selection
        } else if self.corroboration_tasks.contains(task) {
            TaskSet::Corroboration
        } else {
            TaskSet::NonHoldout
        }
    }

    fn harness_mut(&mut self, id: HarnessId) -> Option<&mut Harness> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.harnesses.get_mut(index))
    }

    fn harness_in_state(&mut self, id: HarnessId, state: HarnessState) -> Option<&mut Harness> {
        self.harness_mut(id)
            .filter(|harness| harness.state() == state)
    }

    fn being_created(&mut self, id: HarnessId, in_an_experiment: bool) -> Option<&mut Harness> {
        self.harness_in_state(id, HarnessState::BeingCreated)
            .filter(|harness| harness.is_a_singleton_control_group_harness() == in_an_experiment)
    }
}

fn state_after(outcome: CreationOutcome) -> HarnessState {
    match outcome {
        CreationOutcome::Created => HarnessState::Created,
        CreationOutcome::Unchanged => HarnessState::Unchanged,
        CreationOutcome::Failed | CreationOutcome::BuildFailed | CreationOutcome::TimedOut => {
            HarnessState::Failed
        }
    }
}

#[cfg(test)]
mod tests;
