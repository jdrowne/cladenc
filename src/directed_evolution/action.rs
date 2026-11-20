use std::io;

use super::Shared;
use crate::containers::{self, Attempted, ContainerError, Creation};
use crate::genealogy::{Experiment, HarnessId};
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;

pub(super) enum Ended {
    Procreated {
        child: HarnessId,
        progenitor: HarnessId,
        creation: Creation,
    },
    Established {
        harness: HarnessId,
        experiment: Experiment,
        creation: Creation,
    },
    Attempted {
        harness: HarnessId,
        task: String,
        attempted: Attempted,
    },
    Stopped,
    Failed {
        action: Started,
        error: ContainerError,
    },
}

pub(super) fn carry_out(shared: &Shared<'_>, action: Started, again: bool) -> Ended {
    let outputs_directory = &shared.outputs_directory;
    let cleared = if again {
        discard(outputs_directory, &action)
    } else {
        Ok(())
    };
    let cleared = cleared.map_err(ContainerError::Io);
    let result = match action.clone() {
        Started::Procreation { child, progenitor } => cleared
            .and_then(|()| {
                containers::run_procreation(
                    outputs_directory,
                    &shared.launch,
                    child,
                    progenitor,
                    &shared.halt,
                )
            })
            .map(|creation| Ended::Procreated {
                child,
                progenitor,
                creation,
            }),
        Started::Baseline {
            harness,
            child,
            progenitor,
        } => {
            let experiment = Experiment { child, progenitor };
            cleared
                .and_then(|()| {
                    containers::run_baseline(
                        outputs_directory,
                        &shared.launch,
                        harness,
                        experiment,
                        &shared.halt,
                    )
                })
                .map(|creation| Ended::Established {
                    harness,
                    experiment,
                    creation,
                })
        }
        Started::Attempt { harness, task } => cleared
            .and_then(|()| {
                containers::run_attempt(
                    outputs_directory,
                    &shared.launch,
                    harness,
                    &task,
                    &shared.halt,
                )
            })
            .map(|attempted| Ended::Attempted {
                harness,
                task,
                attempted,
            }),
    };
    match result {
        Ok(ended) => ended,
        Err(ContainerError::Stopped) => Ended::Stopped,
        Err(error) => Ended::Failed { action, error },
    }
}

fn discard(outputs_directory: &OutputsDirectory, action: &Started) -> io::Result<()> {
    match action {
        Started::Procreation { child, .. } => outputs_directory.discard_procreation(*child),
        Started::Baseline { harness, .. } => outputs_directory.discard_baseline(*harness),
        Started::Attempt { harness, task } => outputs_directory.discard_attempt(*harness, task),
    }
}
