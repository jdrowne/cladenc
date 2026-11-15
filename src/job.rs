mod build;
mod container;
mod image;
mod program;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};
use crate::model::{self, Model};
use crate::workspace::{Workspace, holds_only_harness_files};

pub use build::build_progenitor;
use container::{Container, Mount};
pub use image::build_image;
use program::{Ending, deadline, run};

pub const INSTRUCTION: &str = "You are an agentic coding harness. The current directory holds a \
    copy of your own source code. Improve that source code, so that the harness it makes solves \
    more coding tasks than you do. /evaluations holds one directory for each task you have been \
    tried on: the file named outcome in it reads success, failure or timed_out, and the rest is \
    what that attempt left. Use them to find what to improve. Change files in the current \
    directory only: what it holds when you end becomes a new harness, which is tried on tasks in \
    its turn.";

const HARNESS: &str = "/harness";
const WORK: &str = "/work";
const EVALUATIONS: &str = "/evaluations";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch<'a> {
    pub image: &'a str,
    pub run_command: &'a str,
    pub build_command: Option<&'a str>,
    pub model: Model<'a>,
}

#[derive(Debug)]
pub enum JobError {
    Stopped,
    Io(io::Error),
    Docker(PathBuf),
    Exit(i32),
    Signal(i32),
    TimeLimit,
    OtherFiles,
}

impl Display for JobError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped => write!(formatter, "was stopped"),
            Self::Io(error) => write!(formatter, "could not be carried out: {error}"),
            Self::Docker(said) => write!(
                formatter,
                "could not be carried out: Docker did not start its container, and {} holds \
                 what Docker said",
                said.display()
            ),
            Self::Exit(status) => write!(formatter, "exited with status {status}"),
            Self::Signal(signal) => write!(formatter, "was ended by signal {signal}"),
            Self::TimeLimit => write!(formatter, "ran past its time limit"),
            Self::OtherFiles => write!(
                formatter,
                "left something that is not a regular file, a directory or a symbolic link"
            ),
        }
    }
}

impl Error for JobError {}

pub fn run_procreation(
    workspace: &Workspace,
    launch: &Launch<'_>,
    child: HarnessId,
    parent: HarnessId,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<ProcreationOutcome, JobError> {
    workspace
        .start_procreation(child, parent)
        .map_err(JobError::Io)?;
    let job = workspace.running_procreation(child);
    let source = job.join("parent");
    let made = job.join("child");
    let evaluations = workspace.evaluations(parent);
    let deadline = deadline(time_limit);
    let environment: Vec<(&str, &str)> = [
        ("CLADENC_HARNESS", HARNESS),
        ("CLADENC_INSTRUCTION", INSTRUCTION),
    ]
    .into_iter()
    .chain(launch.model.environment())
    .collect();
    let container = Container {
        image: launch.image,
        command: launch.run_command,
        mounts: &[
            Mount {
                from: &source,
                to: HARNESS,
                read_only: false,
            },
            Mount {
                from: &made,
                to: WORK,
                read_only: false,
            },
            Mount {
                from: &evaluations,
                to: EVALUATIONS,
                read_only: true,
            },
        ],
        working_directory: WORK,
        environment: &environment,
    };
    let outcome = match container.run(&job, deadline, stop)? {
        Ending::Exited(0) if holds_only_harness_files(&made).unwrap_or(false) => {
            build::build_child(workspace, launch, child, &made, deadline, stop)?
        }
        Ending::Exited(_) => ProcreationOutcome::Failed,
        Ending::TimedOut => ProcreationOutcome::TimedOut,
    };
    workspace
        .end_procreation(child, outcome)
        .map(|()| outcome)
        .map_err(JobError::Io)
}

pub fn run_evaluation(
    workspace: &Workspace,
    evaluator: &Path,
    model: &Model<'_>,
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
        .arg(job.join("output"))
        .current_dir(job.join("output"))
        .env_remove(model::KEY)
        .envs(model.environment());
    let outcome = match run(command, &job, deadline(time_limit), stop)? {
        Ending::Exited(0) => EvaluationOutcome::Success,
        Ending::Exited(1) => EvaluationOutcome::Failure,
        Ending::Exited(status) => return Err(JobError::Exit(status)),
        Ending::TimedOut => EvaluationOutcome::TimedOut,
    };
    workspace
        .end_evaluation(harness, task, outcome)
        .map(|()| outcome)
        .map_err(JobError::Io)
}
