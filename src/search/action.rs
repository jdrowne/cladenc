use super::Shared;
use crate::containers::{self, ContainerError};
use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};
use crate::journal::Started;

pub(super) enum Ended {
    Procreation {
        child: HarnessId,
        parent: HarnessId,
        result: Result<ProcreationOutcome, ContainerError>,
    },
    Evaluation {
        harness: HarnessId,
        task: String,
        result: Result<EvaluationOutcome, ContainerError>,
    },
}

pub(super) fn carry_out(shared: &Shared<'_>, action: Started, again: bool) -> Ended {
    let settings = shared.settings;
    match action {
        Started::Procreation { child, parent } => {
            let cleared = if again {
                shared.workspace.discard_procreation(child)
            } else {
                Ok(())
            };
            let result = cleared.map_err(ContainerError::Io).and_then(|()| {
                containers::run_procreation(
                    &shared.workspace,
                    &shared.launch,
                    child,
                    parent,
                    settings.procreation_timeout,
                    &shared.halt,
                )
            });
            Ended::Procreation {
                child,
                parent,
                result,
            }
        }
        Started::Evaluation { harness, task } => {
            let cleared = if again {
                shared.workspace.discard_evaluation(harness, &task)
            } else {
                Ok(())
            };
            let result = cleared.map_err(ContainerError::Io).and_then(|()| {
                containers::run_evaluation(
                    &shared.workspace,
                    &shared.launch,
                    harness,
                    &task,
                    settings.evaluation_timeout,
                    &shared.halt,
                )
            });
            Ended::Evaluation {
                harness,
                task,
                result,
            }
        }
    }
}
