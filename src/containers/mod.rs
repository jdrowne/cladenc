mod build;
mod container;
mod grader;
mod image;
mod left;
mod program;
mod shown;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, HarnessId};
use crate::model::Model;
use crate::outputs_directory::{OutputsDirectory, holds_only_harness_files};

pub use build::build_primogenitor;
use container::{Container, Mount};
pub use image::build_image;
pub use left::remove_left_behind;
use program::{Ending, Limits, seconds_since};

pub const INSTRUCTION: &str = "You are an agentic coding harness. The current directory holds a \
    copy of your own source code. Improve that source code, so that the harness it makes \
    completes more coding tasks than you do. /var/log/cladenc/attempts holds the attempts you are \
    shown: one directory for each benchmark, and in it one directory for each task. In the \
    directory of a task, the file named specification says what you were asked to do, the file \
    named result reads completed, not_completed or timed_out, the directory named work holds \
    the workspace of the task as you left it, the files named stdout and stderr hold what you \
    printed, and the directory named grader holds what the grader of the task printed. \
    /var/log/cladenc/line holds one directory for each harness you descend from, named by its \
    id, and /var/log/cladenc/children one for each harness you made before, created or not. In \
    the directory of such a harness, the directory named harness holds its source code, the \
    directory named attempts holds its attempts, laid out as yours are, the directory named \
    procreation holds what the harness that made it printed, in the files named stdout and \
    stderr, with the id of that harness in the file named progenitor and one word in the file \
    named outcome, created, failed, build_failed or timed_out, and the directory named build \
    holds what its build printed. A directory that would hold nothing is not there. Use all of \
    them to find what to improve, and to see what was changed before and what came of it. Make \
    a change to how the harness works on any task: one that \
    answers something you saw in several of your attempts, not in one. Leave nothing in the \
    source code that names a task, a file of a task or the answer to one. Taking out what does \
    not help is an improvement as well. Change files in the current directory only: what it \
    holds when you end becomes a new harness, which attempts tasks in its turn. It may then be \
    asked, as you are now, to improve a copy of its own source code, so a change that makes it \
    better at that counts as well. The new harness is judged by tasks that you are not shown.";

const HARNESS: &str = "/var/lib/cladenc/harness";
const WORK: &str = "/var/lib/cladenc/work";
const ATTEMPTS: &str = "/var/log/cladenc/attempts";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch<'a> {
    pub image: &'a str,
    pub run_command: &'a str,
    pub build_command: Option<&'a str>,
    pub model: Model<'a>,
    pub procreation_timeout: u64,
    pub attempt_timeout: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Creation {
    pub outcome: CreationOutcome,
    pub seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempted {
    pub result: AttemptResult,
    pub seconds: f64,
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
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    child: HarnessId,
    progenitor: HarnessId,
    stop: &AtomicBool,
) -> Result<Creation, ContainerError> {
    outputs_directory
        .start_procreation(child, progenitor)
        .map_err(ContainerError::Io)?;
    let running = outputs_directory.running_procreation(child);
    let source = running.join("progenitor");
    let made = running.join("child");
    let attempts = outputs_directory.attempts(progenitor);
    let shown = shown::to(outputs_directory, progenitor, None);
    let changing = Changing {
        harness: child,
        running: &running,
        source: &source,
        made: &made,
        attempts: &attempts,
        shown: &shown,
    };
    let creation = changing.run(outputs_directory, launch, stop)?;
    outputs_directory
        .end_procreation(child, progenitor, creation.outcome)
        .map(|()| creation)
        .map_err(ContainerError::Io)
}

pub fn run_baseline(
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    harness: HarnessId,
    experiment: Experiment,
    stop: &AtomicBool,
) -> Result<Creation, ContainerError> {
    outputs_directory
        .start_baseline(harness, experiment.progenitor)
        .map_err(ContainerError::Io)?;
    let running = outputs_directory.running_baseline(harness);
    let source = running.join("primogenitor");
    let made = running.join("singleton-control-group-harness");
    let attempts = outputs_directory.attempts(experiment.progenitor);
    let shown = shown::to(
        outputs_directory,
        experiment.progenitor,
        Some(experiment.child),
    );
    let changing = Changing {
        harness,
        running: &running,
        source: &source,
        made: &made,
        attempts: &attempts,
        shown: &shown,
    };
    let creation = changing.run(outputs_directory, launch, stop)?;
    outputs_directory
        .end_baseline(harness, experiment, creation.outcome)
        .map(|()| creation)
        .map_err(ContainerError::Io)
}

struct Changing<'a> {
    harness: HarnessId,
    running: &'a Path,
    source: &'a Path,
    made: &'a Path,
    attempts: &'a Path,
    shown: &'a [(PathBuf, String)],
}

impl Changing<'_> {
    fn run(
        &self,
        outputs_directory: &OutputsDirectory,
        launch: &Launch<'_>,
        stop: &AtomicBool,
    ) -> Result<Creation, ContainerError> {
        let mounts: Vec<Mount<'_>> = [
            Mount {
                from: self.source,
                to: HARNESS,
                read_only: false,
            },
            Mount {
                from: self.made,
                to: WORK,
                read_only: false,
            },
            Mount {
                from: self.attempts,
                to: ATTEMPTS,
                read_only: true,
            },
        ]
        .into_iter()
        .chain(self.shown.iter().map(|(kept, to)| Mount {
            from: kept,
            to,
            read_only: true,
        }))
        .collect();
        let limits = Limits::starting_now(launch.procreation_timeout, stop);
        let environment: Vec<(&str, &str)> = [
            ("CLADENC_HARNESS", HARNESS),
            ("CLADENC_INSTRUCTION", INSTRUCTION),
        ]
        .into_iter()
        .chain(launch.model.environment())
        .collect();
        let container = Container {
            outputs_directory,
            image: launch.image,
            command: launch.run_command,
            mounts: &mounts,
            working_directory: WORK,
            environment: &environment,
        };
        let started = Instant::now();
        let ending = container.run(self.running, limits)?;
        let seconds = seconds_since(started);
        let outcome = match ending {
            Ending::Exited(0) if holds_only_harness_files(self.made).unwrap_or(false) => {
                build::build_child(outputs_directory, launch, self.harness, self.made, limits)?
            }
            Ending::Exited(_) => CreationOutcome::Failed,
            Ending::TimedOut => CreationOutcome::TimedOut,
        };
        Ok(Creation { outcome, seconds })
    }
}

pub fn run_attempt(
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    harness: HarnessId,
    task: &str,
    stop: &AtomicBool,
) -> Result<Attempted, ContainerError> {
    let running = outputs_directory.running_attempt(harness, task);
    let specification = outputs_directory
        .start_attempt(harness, task)
        .and_then(|()| fs::read_to_string(running.join("specification")))
        .map_err(ContainerError::Io)?;
    let source = running.join("harness");
    let work = running.join("work");
    let limits = Limits::starting_now(launch.attempt_timeout, stop);
    let environment: Vec<(&str, &str)> = [
        ("CLADENC_HARNESS", HARNESS),
        ("CLADENC_INSTRUCTION", specification.as_str()),
    ]
    .into_iter()
    .chain(launch.model.environment())
    .collect();
    let container = Container {
        outputs_directory,
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
    let started = Instant::now();
    let ending = container.run(&running, limits)?;
    let seconds = seconds_since(started);
    let result = match ending {
        Ending::Exited(_) => grader::run(outputs_directory, launch.image, task, &running, limits)?,
        Ending::TimedOut => AttemptResult::TimedOut,
    };
    outputs_directory
        .end_attempt(harness, task, result)
        .map(|()| Attempted { result, seconds })
        .map_err(ContainerError::Io)
}

#[cfg(test)]
mod tests;
