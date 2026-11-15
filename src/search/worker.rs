use super::Shared;
use crate::genealogy::{EvaluationOutcome, ExpansionOutcome, HarnessId};
use crate::job::{self, JobError};
use crate::journal::Started;

pub(super) enum Ended {
    Expansion {
        child: HarnessId,
        result: Result<ExpansionOutcome, JobError>,
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
        Started::Expansion { child, parent } => {
            let cleared = if again {
                shared.workspace.discard_expansion(child)
            } else {
                Ok(())
            };
            let result = cleared.map_err(JobError::Io).and_then(|()| {
                job::run_expansion(
                    &shared.workspace,
                    &settings.expander,
                    child,
                    parent,
                    settings.expansion_timeout,
                    &shared.halt,
                )
            });
            Ended::Expansion { child, result }
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
                    &settings.evaluator,
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
