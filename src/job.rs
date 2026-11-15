mod program;

use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use crate::genealogy::{EvaluationOutcome, ExpansionOutcome, HarnessId};
use crate::workspace::{Workspace, holds_only_harness_files};

use program::{Ending, run};

#[derive(Debug)]
pub enum JobError {
    Stopped,
    Io(io::Error),
    Exit(i32),
    Signal(i32),
}

pub fn run_expansion(
    workspace: &Workspace,
    expander: &Path,
    child: HarnessId,
    parent: HarnessId,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<ExpansionOutcome, JobError> {
    workspace
        .start_expansion(child, parent)
        .map_err(JobError::Io)?;
    let job = workspace.running_expansion(child);
    let made = job.join("child");
    let mut command = Command::new(expander);
    command
        .arg(job.join("parent"))
        .arg(&made)
        .arg(workspace.evaluations(parent))
        .arg(job.join("output"));
    let outcome = match run(command, &job, time_limit, stop)? {
        Ending::Passed if holds_only_harness_files(&made).unwrap_or(false) => {
            ExpansionOutcome::Created
        }
        Ending::Passed | Ending::Failed => ExpansionOutcome::Failed,
        Ending::TimedOut => ExpansionOutcome::TimedOut,
    };
    workspace
        .end_expansion(child, outcome)
        .map(|()| outcome)
        .map_err(JobError::Io)
}

pub fn run_evaluation(
    workspace: &Workspace,
    evaluator: &Path,
    harness: HarnessId,
    task: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<EvaluationOutcome, JobError> {
    workspace
        .start_evaluation(harness, task)
        .map_err(JobError::Io)?;
    let job = workspace.running_evaluation(harness, task);
    let mut command = Command::new(evaluator);
    command
        .arg(job.join("harness"))
        .arg(task)
        .arg(job.join("output"));
    let outcome = match run(command, &job, time_limit, stop)? {
        Ending::Passed => EvaluationOutcome::Success,
        Ending::Failed => EvaluationOutcome::Failure,
        Ending::TimedOut => EvaluationOutcome::TimedOut,
    };
    workspace
        .end_evaluation(harness, task, outcome)
        .map(|()| outcome)
        .map_err(JobError::Io)
}
