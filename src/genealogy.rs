mod error;
mod harness;

use serde::{Deserialize, Serialize};

pub use error::GenealogyError;
pub use harness::{Harness, HarnessState};

pub type HarnessId = u64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOutcome {
    Created,
    Failed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationOutcome {
    Success,
    Failure,
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
        Self {
            harnesses: vec![Harness::new(0, None, HarnessState::Created)],
            started: 0,
        }
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

    pub fn start_mutation(&mut self, parent: HarnessId) -> Result<HarnessId, GenealogyError> {
        if self
            .harness_in_state(parent, HarnessState::Created)
            .is_none()
        {
            return Err(GenealogyError::ParentNotCreated { parent });
        }
        let child = self.harness_count();
        self.harnesses.push(Harness::new(
            child,
            Some(parent),
            HarnessState::BeingCreated,
        ));
        Ok(child)
    }

    pub fn end_mutation(
        &mut self,
        child: HarnessId,
        outcome: MutationOutcome,
    ) -> Result<(), GenealogyError> {
        let created = self
            .harness_in_state(child, HarnessState::BeingCreated)
            .ok_or(GenealogyError::ChildNotBeingCreated { child })?;
        created.set_state(match outcome {
            MutationOutcome::Created => HarnessState::Created,
            MutationOutcome::Failed | MutationOutcome::TimedOut => HarnessState::Failed,
        });
        Ok(())
    }

    pub fn start_evaluation(
        &mut self,
        harness: HarnessId,
        task: &str,
    ) -> Result<(), GenealogyError> {
        let evaluated = self
            .harness_in_state(harness, HarnessState::Created)
            .ok_or(GenealogyError::HarnessNotCreated { harness })?;
        if !evaluated.give(task) {
            return Err(GenealogyError::TaskAlreadyGiven {
                harness,
                task: task.to_owned(),
            });
        }
        self.started += 1;
        Ok(())
    }

    pub fn end_evaluation(
        &mut self,
        harness: HarnessId,
        task: &str,
        outcome: EvaluationOutcome,
    ) -> Result<(), GenealogyError> {
        let finished = self
            .harness_mut(harness)
            .is_some_and(|evaluated| evaluated.finish(task, outcome));
        if !finished {
            return Err(GenealogyError::EvaluationNotRunning {
                harness,
                task: task.to_owned(),
            });
        }
        let mut member = Some(harness);
        while let Some(ancestor) = member.and_then(|id| self.harness_mut(id)) {
            ancestor.count_in_clade(outcome);
            member = ancestor.parent();
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
