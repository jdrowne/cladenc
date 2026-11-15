use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::HarnessId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenealogyError {
    ParentNotCreated { parent: HarnessId },
    ChildNotBeingCreated { child: HarnessId },
    HarnessNotCreated { harness: HarnessId },
    TaskAlreadyGiven { harness: HarnessId, task: String },
    EvaluationNotRunning { harness: HarnessId, task: String },
}

impl Display for GenealogyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParentNotCreated { parent } => write!(
                formatter,
                "harness {parent} cannot be expanded because it is not a created harness"
            ),
            Self::ChildNotBeingCreated { child } => {
                write!(formatter, "harness {child} is not being created")
            }
            Self::HarnessNotCreated { harness } => write!(
                formatter,
                "harness {harness} cannot be evaluated because it is not a created harness"
            ),
            Self::TaskAlreadyGiven { harness, task } => write!(
                formatter,
                "harness {harness} has already been given task {task}"
            ),
            Self::EvaluationNotRunning { harness, task } => write!(
                formatter,
                "no evaluation of harness {harness} on task {task} is running"
            ),
        }
    }
}

impl Error for GenealogyError {}
