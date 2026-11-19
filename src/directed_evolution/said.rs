use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::path::Path;

use crate::containers::ContainerError;
use crate::genealogy::{AttemptResult, HarnessId, ProcreationOutcome};
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;

pub(super) enum Said<'a> {
    Making {
        outputs_directory: &'a Path,
        primogenitor: &'a Path,
    },
    CopyingTasks {
        benchmark: &'a Path,
        tasks: u64,
    },
    HoldingOut {
        benchmark: &'a Path,
        selection: u64,
        corroboration: u64,
        tasks: u64,
    },
    BuildingImage {
        image: &'a str,
        dockerfile: &'a Path,
    },
    ImageBuilt {
        image: &'a str,
        outputs_directory: &'a OutputsDirectory,
    },
    ImageNotBuilt {
        error: &'a ContainerError,
        outputs_directory: &'a OutputsDirectory,
    },
    Removed {
        containers: u64,
    },
    Continuing {
        outputs_directory: &'a Path,
        procreations: u64,
        max_procreations: u64,
        harnesses: u64,
    },
    HadFinished {
        best_harness: HarnessId,
        directory: &'a Path,
    },
    Building,
    Built,
    BuildFailed {
        error: &'a ContainerError,
        outputs_directory: &'a OutputsDirectory,
    },
    Started {
        action: &'a Started,
        again: bool,
        max_procreations: u64,
    },
    Procreated {
        child: HarnessId,
        progenitor: HarnessId,
        outcome: ProcreationOutcome,
        outputs_directory: &'a OutputsDirectory,
    },
    Attempted {
        harness: HarnessId,
        task: &'a str,
        result: AttemptResult,
        outputs_directory: &'a OutputsDirectory,
    },
    Failed {
        action: &'a Started,
        error: &'a ContainerError,
    },
    Stopping {
        asked: bool,
        running: u64,
    },
    Finished {
        attempts: u64,
        harnesses: u64,
        best_harness: HarnessId,
        directory: &'a Path,
    },
}

impl Display for Said<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Making {
                outputs_directory,
                primogenitor,
            } => write!(
                formatter,
                "making the outputs directory {} with a copy of the primogenitor {}",
                outputs_directory.display(),
                primogenitor.display()
            ),
            Self::CopyingTasks { benchmark, tasks } => write!(
                formatter,
                "copying the {} of the benchmark {} into the outputs directory",
                counted(*tasks, "task", "tasks"),
                benchmark.display()
            ),
            Self::HoldingOut {
                benchmark,
                selection,
                corroboration,
                tasks,
            } => write!(
                formatter,
                "holding out {} of the {} of the benchmark {} from every progenitor: {} and {}",
                selection + corroboration,
                counted(*tasks, "task", "tasks"),
                benchmark.display(),
                counted(*selection, "selection task", "selection tasks"),
                counted(*corroboration, "corroboration task", "corroboration tasks")
            ),
            Self::BuildingImage { image, dockerfile } => write!(
                formatter,
                "building the image {image} from the Dockerfile {}",
                dockerfile.display()
            ),
            Self::ImageBuilt {
                image,
                outputs_directory,
            } => write!(
                formatter,
                "the image {image} is built: what Docker printed is kept in {}",
                outputs_directory.image().display()
            ),
            Self::ImageNotBuilt {
                error,
                outputs_directory,
            } => write!(
                formatter,
                "the build of the image {error}: what Docker printed is kept in {}",
                outputs_directory.image().display()
            ),
            Self::Removed { containers } => write!(
                formatter,
                "removed {} that the directed evolution left running",
                counted(*containers, "container", "containers")
            ),
            Self::Continuing {
                outputs_directory,
                procreations,
                max_procreations,
                harnesses,
            } => write!(
                formatter,
                "continuing the directed evolution in {}, which has started {procreations} of {} \
                 and holds {}",
                outputs_directory.display(),
                counted(*max_procreations, "procreation", "procreations"),
                counted(*harnesses, "harness", "harnesses")
            ),
            Self::HadFinished {
                best_harness,
                directory,
            } => write!(
                formatter,
                "the directed evolution had finished: its best harness is harness \
                 {best_harness}, kept in {}",
                directory.display()
            ),
            Self::Building => write!(formatter, "building the primogenitor"),
            Self::Built => write!(formatter, "the primogenitor is built"),
            Self::BuildFailed {
                error,
                outputs_directory,
            } => write!(
                formatter,
                "the build of the primogenitor {error}: what it printed is kept in {}",
                outputs_directory.build(0).display()
            ),
            Self::Started {
                action,
                again,
                max_procreations,
            } => match (action, again) {
                (Started::Procreation { child, .. }, false) => write!(
                    formatter,
                    "started {action}, procreation {child} of {max_procreations}"
                ),
                (_, false) => write!(formatter, "started {action}"),
                (_, true) => write!(formatter, "started again {action}"),
            },
            Self::Procreated {
                child,
                progenitor,
                outcome,
                outputs_directory,
            } => {
                let (how, kept) = match outcome {
                    ProcreationOutcome::Created => (
                        format!("created harness {child}, kept in"),
                        outputs_directory.harness(*child),
                    ),
                    ProcreationOutcome::Failed => (
                        "failed: what the harness printed is kept in".to_owned(),
                        outputs_directory.procreation(*child),
                    ),
                    ProcreationOutcome::BuildFailed => (
                        "made a harness that did not build: what its build printed is kept in"
                            .to_owned(),
                        outputs_directory.build(*child),
                    ),
                    ProcreationOutcome::TimedOut => (
                        "ran past its time limit: what the harness printed is kept in".to_owned(),
                        outputs_directory.procreation(*child),
                    ),
                };
                write!(
                    formatter,
                    "the procreation of harness {child} by harness {progenitor} {how} {}",
                    kept.display()
                )
            }
            Self::Attempted {
                harness,
                task,
                result,
                outputs_directory,
            } => {
                let (how, why) = match result {
                    AttemptResult::Completed => ("completed", ""),
                    AttemptResult::NotCompleted => ("did not complete", ""),
                    AttemptResult::TimedOut => (
                        "did not complete",
                        ", as the attempt ran past its time limit",
                    ),
                };
                write!(
                    formatter,
                    "harness {harness} {how} task {task}{why}: what the attempt left is kept in {}",
                    outputs_directory.attempt(*harness, task).display()
                )
            }
            Self::Failed { action, error } => write!(formatter, "{action} {error}"),
            Self::Stopping { asked, running } => {
                let why = if *asked {
                    "asked to stop"
                } else {
                    "stopping the directed evolution"
                };
                write!(
                    formatter,
                    "{why}, with {} running",
                    counted(*running, "container", "containers")
                )
            }
            Self::Finished {
                attempts,
                harnesses,
                best_harness,
                directory,
            } => write!(
                formatter,
                "the directed evolution finished with {} started and {}: its best harness is \
                 harness {best_harness}, kept in {}",
                counted(*attempts, "attempt", "attempts"),
                counted(*harnesses, "harness", "harnesses"),
                directory.display()
            ),
        }
    }
}

pub(super) fn say(writer: &mut dyn Write, said: &Said<'_>) {
    let _ = writeln!(writer, "cladenc: {said}");
}

fn counted(count: u64, one: &str, several: &str) -> String {
    if count == 1 {
        format!("1 {one}")
    } else {
        format!("{count} {several}")
    }
}
