mod error;
mod progress;
mod worker;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::genealogy::HarnessId;
use crate::job::{self, JobError, Launch};
use crate::journal::{self, Journal, Record, Settings, State};
use crate::workspace::Workspace;

pub use error::SearchError;
use progress::Progress;

const WAIT: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Found {
    pub best_harness: HarnessId,
    pub directory: PathBuf,
}

struct Shared<'a> {
    settings: &'a Settings,
    launch: Launch<'a>,
    workspace: Workspace,
    halt: AtomicBool,
}

pub fn search(
    settings: &Settings,
    run_directory: &Path,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, SearchError> {
    let key = settings.model_key().map_err(SearchError::Key)?;
    let (workspace, journal) = Workspace::create(run_directory, &settings.progenitor)
        .and_then(|workspace| {
            workspace
                .copy_benchmark(&settings.benchmark, &settings.tasks)
                .map(|()| workspace)
        })
        .map_err(SearchError::Io)
        .and_then(|workspace| {
            Journal::open(run_directory)
                .map(|journal| (workspace, journal))
                .map_err(SearchError::Journal)
        })?;
    let mut progress = Progress::new(settings, journal, writer);
    progress.record(&Record::Run {
        format: journal::FORMAT,
        settings: Box::new(settings.clone()),
    });
    carry_on(settings, key.as_deref(), workspace, progress, stop)
}

pub fn resume(
    run_directory: &Path,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, SearchError> {
    let (journal, state) = Journal::open_existing(run_directory)
        .and_then(|journal| journal::read(run_directory).map(|state| (journal, state)))
        .map_err(SearchError::Unusable)?;
    let State {
        settings,
        genealogy,
        unfinished,
        next_job,
        best_harness,
    } = state;
    let workspace = Workspace::at(run_directory);
    match best_harness {
        Some(best_harness) => Ok(Found {
            best_harness,
            directory: workspace.harness(best_harness),
        }),
        None => {
            let key = settings.model_key().map_err(SearchError::Key)?;
            let progress =
                Progress::new(&settings, journal, writer).resumed(genealogy, unfinished, next_job);
            carry_on(&settings, key.as_deref(), workspace, progress, stop)
        }
    }
}

fn carry_on(
    settings: &Settings,
    key: Option<&str>,
    workspace: Workspace,
    mut progress: Progress<'_>,
    stop: &AtomicBool,
) -> Result<Found, SearchError> {
    let launch = settings.launch(key);
    if progress.awaits_build(stop) {
        let time_limit = settings.procreation_timeout;
        job::build_progenitor(&workspace, &launch, time_limit, stop).map_err(
            |error| match error {
                JobError::Stopped => SearchError::Asked,
                error => SearchError::Build(error),
            },
        )?;
        progress.built();
    }
    let shared = Shared {
        settings,
        launch,
        workspace,
        halt: AtomicBool::new(false),
    };
    thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        loop {
            while !progress.halted(stop) && progress.running() < settings.workers {
                let Some((job, again)) = progress.take() else {
                    break;
                };
                progress.hand_over(job, again, &shared, scope, &sender);
            }
            if progress.running() == 0 {
                break;
            }
            if let Ok(ended) = receiver.recv_timeout(WAIT) {
                progress.end(ended);
            }
            if progress.halted(stop) {
                shared.halt.store(true, Ordering::SeqCst);
            }
        }
    });
    progress.finish(&shared.workspace, stop)
}
