use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::HarnessId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenealogyError {
    PrimogenitorNotAwaitingBuild,
    ProgenitorNotCreated { progenitor: HarnessId },
    ChildNotBeingCreated { child: HarnessId },
    HarnessNotCreated { harness: HarnessId },
    TaskAlreadyGiven { harness: HarnessId, task: String },
    AssessmentNotRunning { harness: HarnessId, task: String },
}

impl Display for GenealogyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrimogenitorNotAwaitingBuild => {
                write!(formatter, "the primogenitor is not waiting to be built")
            }
            Self::ProgenitorNotCreated { progenitor } => write!(
                formatter,
                "harness {progenitor} cannot procreate because it is not a created harness"
            ),
            Self::ChildNotBeingCreated { child } => {
                write!(formatter, "harness {child} is not being created")
            }
            Self::HarnessNotCreated { harness } => write!(
                formatter,
                "harness {harness} cannot be assessed because it is not a created harness"
            ),
            Self::TaskAlreadyGiven { harness, task } => write!(
                formatter,
                "harness {harness} has already been given task {task}"
            ),
            Self::AssessmentNotRunning { harness, task } => write!(
                formatter,
                "no assessment of harness {harness} on task {task} is running"
            ),
        }
    }
}

impl Error for GenealogyError {}
