use super::Shared;
use crate::genealogy::{EvaluationOutcome, HarnessId, MutationOutcome};
use crate::job::{self, JobError};
use crate::journal::Started;

pub(super) enum Ended {
    Mutation {
        child: HarnessId,
        parent: HarnessId,
        result: Result<MutationOutcome, JobError>,
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
        Started::Mutation { child, parent } => {
            let cleared = if again {
                shared.workspace.discard_mutation(child)
            } else {
                Ok(())
            };
            let result = cleared.map_err(JobError::Io).and_then(|()| {
                job::run_mutation(
                    &shared.workspace,
                    &settings.mutator,
                    child,
                    parent,
                    settings.mutation_timeout,
                    &shared.halt,
                )
            });
            Ended::Mutation {
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
