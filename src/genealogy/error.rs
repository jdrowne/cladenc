use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::HarnessId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenealogyError {
    PrimogenitorNotAwaitingBuild,
    ProgenitorNotCreated {
        progenitor: HarnessId,
    },
    ChildNotBeingCreated {
        child: HarnessId,
    },
    ExperimentNotAwaitingBaseline {
        child: HarnessId,
        progenitor: HarnessId,
    },
    SingletonControlGroupHarnessNotBeingCreated {
        harness: HarnessId,
    },
    HarnessNotCreated {
        harness: HarnessId,
    },
    TaskAlreadyGiven {
        harness: HarnessId,
        task: String,
    },
    AttemptNotRunning {
        harness: HarnessId,
        task: String,
    },
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
                write!(
                    formatter,
                    "harness {child} is not a child that is being created"
                )
            }
            Self::ExperimentNotAwaitingBaseline { child, progenitor } => write!(
                formatter,
                "no experiment at the procreation of harness {child} by harness {progenitor} is \
                 waiting for its baseline"
            ),
            Self::SingletonControlGroupHarnessNotBeingCreated { harness } => write!(
                formatter,
                "harness {harness} is not a singleton control group harness that is being created"
            ),
            Self::HarnessNotCreated { harness } => write!(
                formatter,
                "harness {harness} cannot attempt a task because it is not a created harness"
            ),
            Self::TaskAlreadyGiven { harness, task } => write!(
                formatter,
                "harness {harness} has already been given task {task}"
            ),
            Self::AttemptNotRunning { harness, task } => write!(
                formatter,
                "no attempt of harness {harness} on task {task} is running"
            ),
        }
    }
}

impl Error for GenealogyError {}
