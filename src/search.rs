mod progress;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::genealogy::HarnessId;
use crate::job::JobError;
use crate::journal::{self, Journal, JournalError, Record, Settings};
use crate::workspace::Workspace;

use progress::Progress;

const WAIT: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub enum SearchError {
    Io(io::Error),
    Journal(JournalError),
    Stopped(Option<JobError>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Found {
    pub best_harness: HarnessId,
    pub directory: PathBuf,
}

struct Shared<'a> {
    settings: &'a Settings,
    workspace: Workspace,
    halt: AtomicBool,
}

pub fn search(
    settings: &Settings,
    run_directory: &Path,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, SearchError> {
    let (workspace, journal) = Workspace::create(run_directory, &settings.progenitor)
        .map_err(SearchError::Io)
        .and_then(|workspace| {
            Journal::open(run_directory)
                .map(|journal| (workspace, journal))
                .map_err(SearchError::Journal)
        })?;
    let shared = Shared {
        settings,
        workspace,
        halt: AtomicBool::new(false),
    };
    let mut progress = Progress::new(settings, journal, writer);
    progress.record(&Record::Run {
        format: journal::FORMAT,
        settings: settings.clone(),
    });
    thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        loop {
            while !progress.halted(stop) && progress.running() < settings.workers {
                let Some(job) = progress.next() else {
                    break;
                };
                progress.start(job, &shared, scope, &sender);
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
