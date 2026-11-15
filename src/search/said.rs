use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::path::Path;

use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};
use crate::job::JobError;
use crate::journal::Started;
use crate::workspace::Workspace;

pub enum Said<'a> {
    Making {
        run_directory: &'a Path,
        progenitor: &'a Path,
    },
    CopyingTasks {
        benchmark: &'a Path,
        tasks: u64,
    },
    BuildingImage {
        image: &'a str,
        dockerfile: &'a Path,
    },
    ImageBuilt {
        image: &'a str,
        workspace: &'a Workspace,
    },
    ImageNotBuilt {
        error: &'a JobError,
        workspace: &'a Workspace,
    },
    Continuing {
        run_directory: &'a Path,
        started: u64,
        budget: u64,
        harnesses: u64,
    },
    HadFinished {
        best_harness: HarnessId,
        directory: &'a Path,
    },
    Building,
    Built,
    BuildFailed {
        error: &'a JobError,
        workspace: &'a Workspace,
    },
    Started {
        job: &'a Started,
        again: bool,
        evaluations: u64,
        budget: u64,
    },
    Procreated {
        child: HarnessId,
        parent: HarnessId,
        outcome: ProcreationOutcome,
        workspace: &'a Workspace,
    },
    Evaluated {
        harness: HarnessId,
        task: &'a str,
        outcome: EvaluationOutcome,
        workspace: &'a Workspace,
    },
    Failed {
        job: &'a Started,
        error: &'a JobError,
    },
    Stopping {
        asked: bool,
        running: u64,
    },
    Finished {
        evaluations: u64,
        harnesses: u64,
        best_harness: HarnessId,
        directory: &'a Path,
    },
}

impl Display for Said<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Making {
                run_directory,
                progenitor,
            } => write!(
                formatter,
                "making the run directory {} with a copy of the progenitor {}",
                run_directory.display(),
                progenitor.display()
            ),
            Self::CopyingTasks { benchmark, tasks } => write!(
                formatter,
                "copying the {} of the benchmark {} into the run directory",
                counted(*tasks, "task", "tasks"),
                benchmark.display()
            ),
            Self::BuildingImage { image, dockerfile } => write!(
                formatter,
                "building the image {image} from the Dockerfile {}",
                dockerfile.display()
            ),
            Self::ImageBuilt { image, workspace } => write!(
                formatter,
                "the image {image} is built: what Docker printed is kept in {}",
                workspace.image().display()
            ),
            Self::ImageNotBuilt { error, workspace } => write!(
                formatter,
                "the build of the image {error}: what Docker printed is kept in {}",
                workspace.image().display()
            ),
            Self::Continuing {
                run_directory,
                started,
                budget,
                harnesses,
            } => write!(
                formatter,
                "continuing the search in {}, which has started {started} of its {budget} \
                 evaluations and holds {}",
                run_directory.display(),
                counted(*harnesses, "harness", "harnesses")
            ),
            Self::HadFinished {
                best_harness,
                directory,
            } => write!(
                formatter,
                "the search had finished: its best harness is harness {best_harness}, kept in {}",
                directory.display()
            ),
            Self::Building => write!(formatter, "building the progenitor"),
            Self::Built => write!(formatter, "the progenitor is built"),
            Self::BuildFailed { error, workspace } => write!(
                formatter,
                "the build of the progenitor {error}: what it printed is kept in {}",
                workspace.build(0).display()
            ),
            Self::Started {
                job,
                again,
                evaluations,
                budget,
            } => match (job, again) {
                (Started::Evaluation { .. }, false) => write!(
                    formatter,
                    "started {job}, evaluation {evaluations} of {budget}"
                ),
                (_, false) => write!(formatter, "started {job}"),
                (_, true) => write!(formatter, "started again {job}"),
            },
            Self::Procreated {
                child,
                parent,
                outcome,
                workspace,
            } => {
                let (how, kept) = match outcome {
                    ProcreationOutcome::Created => (
                        format!("created harness {child}, kept in"),
                        workspace.harness(*child),
                    ),
                    ProcreationOutcome::Failed => (
                        "failed: what the harness printed is kept in".to_owned(),
                        workspace.procreation(*child),
                    ),
                    ProcreationOutcome::BuildFailed => (
                        "made a harness that did not build: what its build printed is kept in"
                            .to_owned(),
                        workspace.build(*child),
                    ),
                    ProcreationOutcome::TimedOut => (
                        "ran past its time limit: what the harness printed is kept in".to_owned(),
                        workspace.procreation(*child),
                    ),
                };
                write!(
                    formatter,
                    "the procreation of harness {child} by harness {parent} {how} {}",
                    kept.display()
                )
            }
            Self::Evaluated {
                harness,
                task,
                outcome,
                workspace,
            } => {
                let how = match outcome {
                    EvaluationOutcome::Success => "succeeded",
                    EvaluationOutcome::Failure => "failed",
                    EvaluationOutcome::TimedOut => "ran past its time limit",
                };
                write!(
                    formatter,
                    "the evaluation of harness {harness} on task {task} {how}: what it left is \
                     kept in {}",
                    workspace.evaluation(*harness, task).display()
                )
            }
            Self::Failed { job, error } => write!(formatter, "{job} {error}"),
            Self::Stopping { asked, running } => {
                let why = if *asked {
                    "asked to stop"
                } else {
                    "stopping the search"
                };
                write!(
                    formatter,
                    "{why}, with {} running",
                    counted(*running, "job", "jobs")
                )
            }
            Self::Finished {
                evaluations,
                harnesses,
                best_harness,
                directory,
            } => write!(
                formatter,
                "the search finished with {} started and {}: its best harness is harness \
                 {best_harness}, kept in {}",
                counted(*evaluations, "evaluation", "evaluations"),
                counted(*harnesses, "harness", "harnesses"),
                directory.display()
            ),
        }
    }
}

pub(super) fn say(writer: &mut Option<&mut dyn Write>, said: &Said<'_>) {
    if let Some(writer) = writer {
        let _ = writeln!(writer, "cladenc: {said}");
    }
}

fn counted(count: u64, one: &str, several: &str) -> String {
    if count == 1 {
        format!("1 {one}")
    } else {
        format!("{count} {several}")
    }
}
