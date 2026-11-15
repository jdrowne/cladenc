use super::Shared;
use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};
use crate::job::{self, JobError};
use crate::journal::Started;

pub(super) enum Ended {
    Procreation {
        child: HarnessId,
        parent: HarnessId,
        result: Result<ProcreationOutcome, JobError>,
    },
    Evaluation {
        harness: HarnessId,
        task: String,
        result: Result<EvaluationOutcome, JobError>,
    },
}

pub(super) fn carry_out(shared: &Shared<'_>, job: Started, again: bool) -> Ended {
    let settings = shared.settings;
    match job {
        Started::Procreation { child, parent } => {
            let cleared = if again {
                shared.workspace.discard_procreation(child)
            } else {
                Ok(())
            };
            let result = cleared.map_err(JobError::Io).and_then(|()| {
                job::run_procreation(
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
            let result = cleared.map_err(JobError::Io).and_then(|()| {
                job::run_evaluation(
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
