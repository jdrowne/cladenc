mod error;
mod harness;

use serde::{Deserialize, Serialize};

pub use error::GenealogyError;
pub use harness::{Harness, HarnessState};

pub type HarnessId = u64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcreationOutcome {
    Created,
    Failed,
    BuildFailed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptResult {
    Completed,
    NotCompleted,
    TimedOut,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Genealogy {
    harnesses: Vec<Harness>,
    started: u64,
}

impl Default for Genealogy {
    fn default() -> Self {
        Self::new()
    }
}

impl Genealogy {
    pub fn new() -> Self {
        Self::of_a_primogenitor(HarnessState::Created)
    }

    pub fn unbuilt() -> Self {
        Self::of_a_primogenitor(HarnessState::BeingCreated)
    }

    fn of_a_primogenitor(state: HarnessState) -> Self {
        Self {
            harnesses: vec![Harness::new(0, None, state)],
            started: 0,
        }
    }

    pub fn awaits_build(&self) -> bool {
        self.harnesses[0].state() == HarnessState::BeingCreated
    }

    pub fn build_primogenitor(&mut self) -> Result<(), GenealogyError> {
        self.harness_in_state(0, HarnessState::BeingCreated)
            .ok_or(GenealogyError::PrimogenitorNotAwaitingBuild)?
            .set_state(HarnessState::Created);
        Ok(())
    }

    pub fn harnesses(&self) -> &[Harness] {
        &self.harnesses
    }

    pub fn harness_count(&self) -> u64 {
        self.harnesses.len() as u64
    }

    pub fn started(&self) -> u64 {
        self.started
    }

    pub fn created_harnesses(&self) -> Vec<&Harness> {
        self.harnesses
            .iter()
            .filter(|harness| harness.state() == HarnessState::Created)
            .collect()
    }

    pub fn start_procreation(
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

    pub fn end_procreation(
        &mut self,
        child: HarnessId,
        outcome: ProcreationOutcome,
    ) -> Result<(), GenealogyError> {
        let created = self
            .harness_in_state(child, HarnessState::BeingCreated)
            .ok_or(GenealogyError::ChildNotBeingCreated { child })?;
        created.set_state(match outcome {
            ProcreationOutcome::Created => HarnessState::Created,
            ProcreationOutcome::Failed
            | ProcreationOutcome::BuildFailed
            | ProcreationOutcome::TimedOut => HarnessState::Failed,
        });
        Ok(())
    }

    pub fn start_attempt(&mut self, harness: HarnessId, task: &str) -> Result<(), GenealogyError> {
        let attempting = self
            .harness_in_state(harness, HarnessState::Created)
            .ok_or(GenealogyError::HarnessNotCreated { harness })?;
        if !attempting.give(task) {
            return Err(GenealogyError::TaskAlreadyGiven {
                harness,
                task: task.to_owned(),
            });
        }
        self.started += 1;
        Ok(())
    }

    pub fn end_attempt(
        &mut self,
        harness: HarnessId,
        task: &str,
        result: AttemptResult,
    ) -> Result<(), GenealogyError> {
        let finished = self
            .harness_mut(harness)
            .is_some_and(|attempting| attempting.finish(task, result));
        if !finished {
            return Err(GenealogyError::AttemptNotRunning {
                harness,
                task: task.to_owned(),
            });
        }
        let mut member = Some(harness);
        while let Some(ancestor) = member.and_then(|id| self.harness_mut(id)) {
            ancestor.count_in_clade(result);
            member = ancestor.progenitor();
        }
        Ok(())
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
}
