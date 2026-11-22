mod build;
mod container;
mod grader;
mod health_check;
mod image;
mod left;
mod program;
mod reward;
mod shown;

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::benchmark::TaskConfiguration;
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, HarnessId};
use crate::model::Model;
use crate::outputs_directory::{OutputsDirectory, holds_only_harness_files, same_tree};

pub use build::build_primogenitor;
use container::{Container, Mount, Run, Shell, Up};
pub use image::{Machine, build_environment, build_image, find_image, machine, remove_images};
pub use left::remove_left_behind;
use program::{Ending, FOREVER, Limits, seconds_since};

pub const INSTRUCTION: &str = "You are an agentic coding harness. The current directory holds a \
    copy of your own source code. Improve that source code, so that the harness it makes completes \
    more coding tasks than you do. /var/log/cladenc/attempts holds the attempts you are shown: one \
    directory for each benchmark, and in it one directory for each task. In the directory of a \
    task, the file named specification says what you were asked to do, the file named result reads \
    completed, not_completed or timed_out, the file named ending reads ended, stopped where \
    you ran past the time limit of the task and were stopped, or not_started where the \
    environment of the task did not pass its health check, so that you were never started and \
    the attempt holds nothing of you or of the grader, the directory named work holds what \
    you left in the \
    working directory of the task, the file named changes lists every path you added, changed or \
    removed in the container of the task, one to a line after A, C or D, the files named stdout and stderr hold what you printed, the \
    directory named grader holds what the grader of the task printed, in the files named stdout \
    and stderr, and the files it left in logs/verifier, and the directory named health-check \
    holds what the health check of the environment of the task printed the last time it ran, in \
    the files named stdout and stderr. Once you had ended or were stopped, the grader ran the \
    tests of the task, which you are not shown, on what you left, and wrote its score in \
    logs/verifier/reward.txt, or one score for each label in logs/verifier/reward.json: the task \
    was completed where every score is 1. /var/log/cladenc/line holds one directory for each \
    harness you descend from, named by its id, and /var/log/cladenc/children one for each harness \
    you made before, created or not. In the directory of such a harness, the directory named \
    harness holds its source code, the directory named attempts holds its attempts, laid out as \
    yours are, the directory named procreation holds what the harness that made it printed, in the \
    files named stdout and stderr, with the id of that harness in the file named progenitor and \
    one word in the file named outcome, created, unchanged, failed, build_failed or timed_out, and \
    the directory named build holds what its build printed. A directory that would hold nothing is \
    not there. Use all of them to find what to improve, and to see what was changed before and \
    what came of it. Make a change to how the harness works on any task: one that answers \
    something you saw in several of your attempts, not in one. Leave nothing in the source code \
    that names a task, a file of a task or the answer to one. Taking out what does not help is an \
    improvement as well. Change files in the current directory only: what it holds when you end \
    becomes a new harness, which attempts tasks in its turn. It may then be asked, as you are now, \
    to improve a copy of its own source code, so a change that makes it better at that counts as \
    well. Where you leave the current directory as it was, no new harness is made: a change that \
    you only print in your answer changes no file. The new harness is judged by tasks that you are \
    not shown.";

const HARNESS: &str = "/var/lib/cladenc/harness";
const WORK: &str = "/var/lib/cladenc/work";
const ATTEMPTS: &str = "/var/log/cladenc/attempts";
const HEALTH_CHECK: &str = "health-check";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch<'a> {
    pub image: &'a str,
    pub run_command: &'a str,
    pub build_command: Option<&'a str>,
    pub model: Model<'a>,
    pub procreation_timeout: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Environment {
    pub image: String,
    pub configuration: TaskConfiguration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempt<'a> {
    pub harness: HarnessId,
    pub task: &'a str,
    pub environment: &'a Environment,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Creation {
    pub outcome: CreationOutcome,
    pub seconds: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessEnding {
    Ended,
    Stopped,
    NotStarted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempted {
    pub result: AttemptResult,
    pub seconds: f64,
    pub ending: HarnessEnding,
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
        copied_from: progenitor,
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
        copied_from: experiment.progenitor,
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
    copied_from: HarnessId,
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
            mounts: &mounts,
            working_directory: WORK,
        };
        let shell = Shell {
            command: launch.run_command,
            environment: &environment,
        };
        let started = Instant::now();
        let ending = container.run(&shell, self.running, limits)?;
        let seconds = seconds_since(started);
        let outcome = match ending {
            Ending::Exited(0) => self.made_of_what_was_left(outputs_directory, launch, limits)?,
            Ending::Exited(_) => CreationOutcome::Failed,
            Ending::TimedOut => CreationOutcome::TimedOut,
        };
        Ok(Creation { outcome, seconds })
    }

    fn made_of_what_was_left(
        &self,
        outputs_directory: &OutputsDirectory,
        launch: &Launch<'_>,
        limits: Limits<'_>,
    ) -> Result<CreationOutcome, ContainerError> {
        let copied = outputs_directory.harness(self.copied_from);
        if !holds_only_harness_files(self.made).unwrap_or(false) {
            Ok(CreationOutcome::Failed)
        } else if same_tree(self.made, &copied).unwrap_or(false) {
            Ok(CreationOutcome::Unchanged)
        } else {
            build::build_child(outputs_directory, launch, self.harness, self.made, limits)
        }
    }
}

pub fn run_attempt(
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    attempt: Attempt<'_>,
    stop: &AtomicBool,
) -> Result<Attempted, ContainerError> {
    let running = outputs_directory.running_attempt(attempt.harness, attempt.task);
    let specification = outputs_directory
        .start_attempt(attempt.harness, attempt.task)
        .and_then(|()| fs::read_to_string(running.join("specification")))
        .map_err(ContainerError::Io)?;
    let attempting = Attempting {
        attempt,
        running: &running,
        specification: &specification,
    };
    let attempted = attempting.run(outputs_directory, launch, stop)?;
    outputs_directory
        .end_attempt(attempt.harness, attempt.task, attempted.result)
        .map(|()| attempted)
        .map_err(ContainerError::Io)
}

struct Attempting<'a> {
    attempt: Attempt<'a>,
    running: &'a Path,
    specification: &'a str,
}

impl Attempting<'_> {
    fn run(
        &self,
        outputs_directory: &OutputsDirectory,
        launch: &Launch<'_>,
        stop: &AtomicBool,
    ) -> Result<Attempted, ContainerError> {
        let environment: Vec<(&str, &str)> = [
            ("CLADENC_HARNESS", HARNESS),
            ("CLADENC_INSTRUCTION", self.specification),
        ]
        .into_iter()
        .chain(launch.model.environment())
        .collect();
        let shell = Shell {
            command: launch.run_command,
            environment: &environment,
        };
        let started = Instant::now();
        let environment = self.attempt.environment;
        let up = Up::start(outputs_directory, environment, self.running)?;
        if !self.is_healthy(&up, stop)? {
            return Ok(self.not_started(started));
        }
        let run = Run {
            shell: &shell,
            user: environment
                .configuration
                .harness_user
                .as_deref()
                .unwrap_or(up.user()),
            variables: &[],
        };
        up.put(
            &outputs_directory.harness(self.attempt.harness),
            HARNESS,
            run.user,
        )?;
        let before: BTreeSet<String> = up.changes().into_iter().collect();
        let time_limit = environment.configuration.harness_time_limit;
        let limits = Limits::within(time_limit.unwrap_or(FOREVER), stop);
        let ending = up.run(&run, self.running, limits)?;
        let seconds = seconds_since(started);
        let (ending, word) = if matches!(ending, Ending::TimedOut) {
            up.end_everything();
            (HarnessEnding::Stopped, "stopped\n")
        } else {
            (HarnessEnding::Ended, "ended\n")
        };
        let _ = fs::write(self.running.join("ending"), word);
        let changes: String = up
            .changes()
            .into_iter()
            .filter(|change| !before.contains(change))
            .map(|change| change + "\n")
            .collect();
        let _ = fs::write(self.running.join("changes"), changes);
        up.keep_work(&self.running.join("work"));
        let result = grader::run(outputs_directory, &up, &self.attempt, self.running, stop)?;
        Ok(Attempted {
            result,
            seconds,
            ending,
        })
    }

    fn is_healthy(&self, up: &Up, stop: &AtomicBool) -> Result<bool, ContainerError> {
        match &self.attempt.environment.configuration.health_check {
            Some(health_check) => {
                let kept = self.running.join(HEALTH_CHECK);
                let _ = fs::create_dir(&kept);
                health_check::passes(up, health_check, &kept, stop)
            }
            None => Ok(true),
        }
    }

    fn not_started(&self, started: Instant) -> Attempted {
        let _ = fs::write(self.running.join("ending"), "not_started\n");
        let _ = fs::remove_file(self.running.join("stderr"));
        let _ = fs::remove_dir_all(self.running.join("grader"));
        Attempted {
            result: AttemptResult::NotCompleted,
            seconds: seconds_since(started),
            ending: HarnessEnding::NotStarted,
        }
    }
}

#[cfg(test)]
mod tests;
