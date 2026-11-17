mod build;
mod container;
mod grader;
mod image;
mod left;
mod program;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::genealogy::{AttemptResult, HarnessId, ProcreationOutcome};
use crate::model::Model;
use crate::workspace::{Workspace, holds_only_harness_files};

pub use build::build_primogenitor;
use container::{Container, Mount};
pub use image::build_image;
pub use left::remove_left_behind;
use program::{Ending, deadline};

pub const INSTRUCTION: &str = "You are an agentic coding harness. The current directory holds a \
    copy of your own source code. Improve that source code, so that the harness it makes \
    completes more coding tasks than you do. /var/log/cladenc/attempts holds one directory for \
    each benchmark, and in it one directory for each task you have attempted. In the directory \
    of a task, the file named specification says what you were asked to do, the file named \
    result reads completed, not_completed or timed_out, the directory named work holds the \
    workspace of the task as you left it, the files named stdout and stderr hold what you \
    printed, and the directory named grader holds what the grader of the task printed. Use them \
    to find what to improve. Change files in the current directory only: what it holds when you \
    end becomes a new harness, which attempts tasks in its turn.";

const HARNESS: &str = "/var/lib/cladenc/harness";
const WORK: &str = "/var/lib/cladenc/work";
const ATTEMPTS: &str = "/var/log/cladenc/attempts";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch<'a> {
    pub image: &'a str,
    pub run_command: &'a str,
    pub build_command: Option<&'a str>,
    pub model: Model<'a>,
}

#[derive(Debug)]
pub enum ContainerError {
    Stopped,
    Io(io::Error),
    Docker(PathBuf),
    Exit(i32),
    Signal(i32),
    TimeLimit,
    OtherFiles,
}

impl Display for ContainerError {
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

impl Error for ContainerError {}

pub fn run_procreation(
    workspace: &Workspace,
    launch: &Launch<'_>,
    child: HarnessId,
    progenitor: HarnessId,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<ProcreationOutcome, ContainerError> {
    workspace
        .start_procreation(child, progenitor)
        .map_err(ContainerError::Io)?;
    let running = workspace.running_procreation(child);
    let source = running.join("progenitor");
    let made = running.join("child");
    let attempts = workspace.attempts(progenitor);
    let deadline = deadline(time_limit);
    let environment: Vec<(&str, &str)> = [
        ("CLADENC_HARNESS", HARNESS),
        ("CLADENC_INSTRUCTION", INSTRUCTION),
    ]
    .into_iter()
    .chain(launch.model.environment())
    .collect();
    let container = Container {
        workspace,
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
                from: &attempts,
                to: ATTEMPTS,
                read_only: true,
            },
        ],
        working_directory: WORK,
        environment: &environment,
    };
    let outcome = match container.run(&running, deadline, stop)? {
        Ending::Exited(0) if holds_only_harness_files(&made).unwrap_or(false) => {
            build::build_child(workspace, launch, child, &made, deadline, stop)?
        }
        Ending::Exited(_) => ProcreationOutcome::Failed,
        Ending::TimedOut => ProcreationOutcome::TimedOut,
    };
    workspace
        .end_procreation(child, outcome)
        .map(|()| outcome)
        .map_err(ContainerError::Io)
}

pub fn run_attempt(
    workspace: &Workspace,
    launch: &Launch<'_>,
    harness: HarnessId,
    task: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<AttemptResult, ContainerError> {
    let running = workspace.running_attempt(harness, task);
    let specification = workspace
        .start_attempt(harness, task)
        .and_then(|()| fs::read_to_string(running.join("specification")))
        .map_err(ContainerError::Io)?;
    let source = running.join("harness");
    let work = running.join("work");
    let deadline = deadline(time_limit);
    let environment: Vec<(&str, &str)> = [
        ("CLADENC_HARNESS", HARNESS),
        ("CLADENC_INSTRUCTION", specification.as_str()),
    ]
    .into_iter()
    .chain(launch.model.environment())
    .collect();
    let container = Container {
        workspace,
        image: launch.image,
        command: launch.run_command,
        mounts: &[
            Mount {
                from: &source,
                to: HARNESS,
                read_only: false,
            },
            Mount {
                from: &work,
                to: WORK,
                read_only: false,
            },
        ],
        working_directory: WORK,
        environment: &environment,
    };
    let result = match container.run(&running, deadline, stop)? {
        Ending::Exited(_) => grader::run(workspace, launch.image, task, &running, deadline, stop)?,
        Ending::TimedOut => AttemptResult::TimedOut,
    };
    workspace
        .end_attempt(harness, task, result)
        .map(|()| result)
        .map_err(ContainerError::Io)
}
