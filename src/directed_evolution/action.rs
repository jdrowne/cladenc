use super::Shared;
use crate::containers::{self, ContainerError};
use crate::genealogy::{AttemptResult, HarnessId, ProcreationOutcome};
use crate::journal::Started;

pub(super) enum Ended {
    Procreation {
        child: HarnessId,
        progenitor: HarnessId,
        result: Result<ProcreationOutcome, ContainerError>,
    },
    Attempt {
        harness: HarnessId,
        task: String,
        result: Result<AttemptResult, ContainerError>,
    },
}

pub(super) fn carry_out(shared: &Shared<'_>, action: Started, again: bool) -> Ended {
    match action {
        Started::Procreation { child, progenitor } => {
            let cleared = if again {
                shared.outputs_directory.discard_procreation(child)
            } else {
                Ok(())
            };
            let result = cleared.map_err(ContainerError::Io).and_then(|()| {
                containers::run_procreation(
                    &shared.outputs_directory,
                    &shared.launch,
                    child,
                    progenitor,
                    &shared.halt,
                )
            });
            Ended::Procreation {
                child,
                progenitor,
                result,
            }
        }
        Started::Attempt { harness, task } => {
            let cleared = if again {
                shared.outputs_directory.discard_attempt(harness, &task)
            } else {
                Ok(())
            };
            let result = cleared.map_err(ContainerError::Io).and_then(|()| {
                containers::run_attempt(
                    &shared.outputs_directory,
                    &shared.launch,
                    harness,
                    &task,
                    &shared.halt,
                )
            });
            Ended::Attempt {
                harness,
                task,
                result,
            }
        }
    }
}
