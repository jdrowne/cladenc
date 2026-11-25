mod artifacts;
mod build;
mod collect;
mod container;
mod docker;
mod harbor;
mod health_check;
mod image;
mod left;
mod program;
mod reward;
mod shown;
mod verifier;

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::benchmark::TaskConfiguration;
use crate::file_system::FileSystem;
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, HarnessId};
use crate::model::Model;
use crate::outputs_directory::{holds_only_harness_files, same_tree};

pub use build::build_primogenitor;
use container::{Bringing, Container, Mount, Run, Shell, Up};
pub use docker::{Docker, Host, Outside};
use health_check::Backoff;
pub use image::{
    Machine, build_environment, build_image, build_verifier_environment, find_image, machine,
    remove_images,
};
pub use left::remove_left_behind;
pub use program::{Ending, Limits, Program};
use program::{FOREVER, seconds_since};
use verifier::Verifying;

pub const INSTRUCTION: &str = "You are an agentic coding harness. The current directory holds a \
    copy of your own source code. Improve that source code, so that the harness it makes completes \
    more coding tasks than you do. /var/log/cladenc/attempts holds the attempts you are shown: one \
    directory for each benchmark, and in it one directory for each task. In the directory of a \
    task, the file named specification says what you were asked to do, the file named result reads \
    completed, not_completed or timed_out, the file named ending reads ended, or stopped where \
    you ran past the time limit of the task and were stopped, the directory named work holds what \
    you left in the \
    working directory of the task, the file named changes lists every path you added, changed or \
    removed in the container of the task, one to a line after A, C or D, the directory named \
    artifacts holds what you left at the paths that the task names to be kept, where it names \
    any, and what you left in /logs/artifacts, in logs/artifacts, the directory named \
    collection-hooks holds what the commands that the task runs before those paths are kept \
    printed, one directory for each, where it runs any, the files named stdout and stderr hold \
    what you printed, the \
    directory named verifier holds what the verifier of the task printed, in the files named \
    stdout and stderr, and the files it left in logs/verifier, and the directory named \
    health-check holds what the health check of the environment of the task printed the last time \
    it ran, in the files named stdout and stderr. Once you had ended or were stopped, the verifier \
    ran the tests of the task, which you are not shown, on what you left, or, where the task has \
    its verifier run in a container of its own, on nothing but what the directory named artifacts \
    holds, and wrote its score in logs/verifier/reward.txt, or one score for each label in \
    logs/verifier/reward.json: the task was completed where every score is 1. \
    /var/log/cladenc/line holds one directory for each harness you descend from, named by its id, \
    and /var/log/cladenc/children one for each harness you made before, created or not. In the \
    directory of such a harness, the directory named harness holds its source code, the directory \
    named attempts holds its attempts, laid out as yours are, the directory named procreation \
    holds what the harness that made it printed, in the files named stdout and stderr, with the id \
    of that harness in the file named progenitor and one word in the file named outcome, created, \
    unchanged, failed, build_failed or timed_out, and the directory named build holds what its \
    build printed. A directory that would hold nothing is not there. Use all of them to find what \
    to improve, and to see what was changed before and what came of it. Make a change to how the \
    harness works on any task: one that answers something you saw in several of your attempts, not \
    in one. Leave nothing in the source code that names a task, a file of a task or the answer to \
    one. Taking out what does not help is an improvement as well. Change files in the current \
    directory only: what it holds when you end becomes a new harness, which attempts tasks in its \
    turn. It may then be asked, as you are now, to improve a copy of its own source code, so a \
    change that makes it better at that counts as well. Where you leave the current directory as \
    it was, no new harness is made: a change that you only print in your answer changes no file. \
    The new harness is judged by tasks that you are not shown.";

const HARNESS: &str = "/var/opt/cladenc/harness";
const WORK: &str = "/var/opt/cladenc/work";
const ATTEMPTS: &str = "/var/log/cladenc/attempts";
const HEALTH_CHECK: &str = "health-check";
const ARTIFACTS: &str = "artifacts";
const COLLECTION_HOOKS: &str = "collection-hooks";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch<'a> {
    pub image: &'a str,
    pub run_command: &'a str,
    pub build_command: Option<&'a str>,
    pub model: Model<'a>,
    pub procreation_timeout: u64,
    pub max_verifier_health_check_retries: u64,
    pub max_task_health_check_retries: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Environment {
    pub image: String,
    pub configuration: TaskConfiguration,
    pub verifier: Option<VerifierImage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifierImage {
    pub name: String,
    pub holds_verifier: bool,
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
    pub(crate) seconds: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HarnessEnding {
    Ended,
    Stopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Checked {
    Task,
    Verifier,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Retry {
    pub(crate) checked: Checked,
    pub(crate) number: u64,
    pub(crate) most: u64,
    pub(crate) wait: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempted {
    pub result: AttemptResult,
    pub(crate) seconds: f64,
    pub(crate) ending: HarnessEnding,
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
    TaskHealthCheck { tries: u64, kept: PathBuf },
    VerifierHealthCheck { tries: u64, kept: PathBuf },
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
            Self::TaskHealthCheck { tries, kept } => write!(
                formatter,
                "was not carried out: the health check of the task did not pass in {tries} {}, \
                 so the harness was not started, and {} holds what the health check printed the \
                 last time it ran",
                if *tries == 1 { "try" } else { "tries" },
                kept.display()
            ),
            Self::VerifierHealthCheck { tries, kept } => write!(
                formatter,
                "was not verified: the health check of its verifier container did not pass \
                 in {tries} {}, so the verifier was not run, and {} holds what the health check \
                 printed the last time it ran",
                if *tries == 1 { "try" } else { "tries" },
                kept.display()
            ),
        }
    }
}

impl Error for ContainerError {}

pub fn run_procreation(
    host: Host<'_>,
    launch: &Launch<'_>,
    child: HarnessId,
    progenitor: HarnessId,
    stop: &AtomicBool,
) -> Result<Creation, ContainerError> {
    let outputs_directory = host.outputs_directory;
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
    let creation = changing.run(host, launch, stop)?;
    outputs_directory
        .end_procreation(child, progenitor, creation.outcome)
        .map(|()| creation)
        .map_err(ContainerError::Io)
}

pub fn run_baseline(
    host: Host<'_>,
    launch: &Launch<'_>,
    harness: HarnessId,
    experiment: Experiment,
    stop: &AtomicBool,
) -> Result<Creation, ContainerError> {
    let outputs_directory = host.outputs_directory;
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
    let creation = changing.run(host, launch, stop)?;
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
        host: Host<'_>,
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
            host,
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
            Ending::Exited(0) => self.made_of_what_was_left(host, launch, limits)?,
            Ending::Exited(_) => CreationOutcome::Failed,
            Ending::TimedOut => CreationOutcome::TimedOut,
        };
        Ok(Creation { outcome, seconds })
    }

    fn made_of_what_was_left(
        &self,
        host: Host<'_>,
        launch: &Launch<'_>,
        limits: Limits<'_>,
    ) -> Result<CreationOutcome, ContainerError> {
        let files = host.outputs_directory.files();
        let copied = host.outputs_directory.harness(self.copied_from);
        if !holds_only_harness_files(files, self.made).unwrap_or(false) {
            Ok(CreationOutcome::Failed)
        } else if same_tree(files, self.made, &copied).unwrap_or(false) {
            Ok(CreationOutcome::Unchanged)
        } else {
            build::build_child(host, launch, self.harness, self.made, limits)
        }
    }
}

pub fn run_attempt(
    host: Host<'_>,
    launch: &Launch<'_>,
    attempt: Attempt<'_>,
    stop: &AtomicBool,
    retrying: &dyn Fn(Retry),
) -> Result<Attempted, ContainerError> {
    let outputs_directory = host.outputs_directory;
    let running = outputs_directory.running_attempt(attempt.harness, attempt.task);
    let specification = outputs_directory
        .start_attempt(attempt.harness, attempt.task)
        .and_then(|()| read_specification(outputs_directory.files(), &running))
        .map_err(ContainerError::Io)?;
    let attempting = Attempting {
        attempt,
        running: &running,
        specification: &specification,
    };
    let attempted = attempting.run(host, launch, stop, retrying)?;
    outputs_directory
        .end_attempt(attempt.harness, attempt.task, attempted.result)
        .map(|()| attempted)
        .map_err(ContainerError::Io)
}

fn read_specification(files: &dyn FileSystem, running: &Path) -> io::Result<String> {
    files.read_text(&running.join("specification"))
}

fn keep(files: &dyn FileSystem, file: &Path, text: &str) {
    let _ = files.write(file, text.as_bytes());
}

fn make_directory(files: &dyn FileSystem, directory: &Path) {
    let _ = files.make_directories(directory);
}

struct Attempting<'a> {
    attempt: Attempt<'a>,
    running: &'a Path,
    specification: &'a str,
}

impl Attempting<'_> {
    fn run(
        &self,
        host: Host<'_>,
        launch: &Launch<'_>,
        stop: &AtomicBool,
        retrying: &dyn Fn(Retry),
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
        let environment = self.attempt.environment;
        let (up, started) = self.brought_up(host, launch, stop, retrying)?;
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
            &host.outputs_directory.harness(self.attempt.harness),
            HARNESS,
            run.user,
        )
        .and_then(|()| harbor::make_artifacts(&up, run.user))?;
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
        let files = host.outputs_directory.files();
        keep(files, &self.running.join("ending"), word);
        let changes: String = up
            .changes()
            .into_iter()
            .filter(|change| !before.contains(change))
            .map(|change| change + "\n")
            .collect();
        keep(files, &self.running.join("changes"), &changes);
        up.keep_work(&self.running.join("work"));
        collect::run(
            &up,
            &environment.configuration.collect_commands,
            &self.running.join(COLLECTION_HOOKS),
            stop,
        )?;
        artifacts::keep(
            &up,
            &environment.configuration.artifacts,
            &self.running.join(ARTIFACTS),
        );
        let verifying = Verifying {
            host,
            attempt: &self.attempt,
            running: self.running,
            stop,
            max_retries: launch.max_verifier_health_check_retries,
            retrying,
        };
        let result = verifying.run(up)?;
        Ok(Attempted {
            result,
            seconds,
            ending,
        })
    }

    fn brought_up<'b>(
        &self,
        host: Host<'b>,
        launch: &Launch<'_>,
        stop: &AtomicBool,
        retrying: &dyn Fn(Retry),
    ) -> Result<(Up<'b>, Instant), ContainerError> {
        let bringing = Bringing::of_the_task(self.attempt.environment);
        let health_check = self.attempt.environment.configuration.health_check.as_ref();
        let kept = self.running.join(HEALTH_CHECK);
        let backoff = Backoff {
            checked: Checked::Task,
            first_wait: health_check.map_or(Duration::ZERO, |health_check| health_check.interval),
            max_retries: launch.max_task_health_check_retries,
            stop,
        };
        let healthy = health_check::retried(&backoff, retrying, || {
            let started = Instant::now();
            let up = Up::start(host, &bringing, self.running)?;
            health_check::passes_where_given(&up, health_check, &kept, stop)
                .map(|passes| passes.then_some((up, started)))
        })?;
        healthy.ok_or(ContainerError::TaskHealthCheck {
            tries: launch.max_task_health_check_retries.saturating_add(1),
            kept,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests;
