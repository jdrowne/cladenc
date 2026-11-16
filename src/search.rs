mod action;
mod error;
mod progress;
mod said;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::containers::{self, ContainerError, Launch};
use crate::genealogy::HarnessId;
use crate::journal::{self, Journal, Record, Settings, State};
use crate::workspace::Workspace;

pub use error::SearchError;
use progress::Progress;
pub use said::Said;
use said::say;

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
    writer: &mut dyn Write,
) -> Result<Found, SearchError> {
    let key = settings.model_key().map_err(SearchError::Key)?;
    let making = Said::Making {
        run_directory,
        progenitor: &settings.progenitor,
    };
    say(writer, &making);
    let (workspace, journal) = Workspace::create(run_directory, &settings.progenitor)
        .and_then(|workspace| {
            settings
                .benchmarks
                .iter()
                .try_for_each(|benchmark| {
                    let copying = Said::CopyingTasks {
                        benchmark: &benchmark.directory,
                        tasks: benchmark.tasks.len() as u64,
                    };
                    say(writer, &copying);
                    workspace.copy_benchmark(benchmark)
                })
                .map(|()| workspace)
        })
        .map_err(SearchError::Io)
        .and_then(|workspace| build_image(settings, &workspace, stop, writer).map(|()| workspace))
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

fn build_image(
    settings: &Settings,
    workspace: &Workspace,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<(), SearchError> {
    let given = settings
        .dockerfile
        .as_deref()
        .zip(settings.docker_build_timeout);
    let Some((dockerfile, time_limit)) = given else {
        return Ok(());
    };
    let image = &settings.docker_image;
    say(writer, &Said::BuildingImage { image, dockerfile });
    match containers::build_image(workspace, dockerfile, image, time_limit, stop) {
        Ok(()) => {
            say(writer, &Said::ImageBuilt { image, workspace });
            Ok(())
        }
        Err(error) => {
            say(
                writer,
                &Said::ImageNotBuilt {
                    error: &error,
                    workspace,
                },
            );
            Err(SearchError::Image(error))
        }
    }
}

pub fn resume(
    run_directory: &Path,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, SearchError> {
    let (journal, state) = Journal::open_existing(run_directory)
        .and_then(|journal| journal::read(run_directory).map(|state| (journal, state)))
        .map_err(SearchError::Unusable)?;
    let State {
        settings,
        genealogy,
        unfinished,
        next_action,
        best_harness,
    } = state;
    let workspace = Workspace::at(run_directory);
    match best_harness {
        Some(best_harness) => {
            let directory = workspace.harness(best_harness);
            let finished = Said::HadFinished {
                best_harness,
                directory: &directory,
            };
            say(writer, &finished);
            Ok(Found {
                best_harness,
                directory,
            })
        }
        None => {
            let removed = containers::remove_left_behind(&workspace).map_err(SearchError::Left)?;
            if removed > 0 {
                say(
                    writer,
                    &Said::Removed {
                        containers: removed,
                    },
                );
            }
            let key = settings.model_key().map_err(SearchError::Key)?;
            let continuing = Said::Continuing {
                run_directory,
                started: genealogy.started(),
                budget: settings.budget,
                harnesses: genealogy.harness_count(),
            };
            say(writer, &continuing);
            let progress = Progress::new(&settings, journal, writer).resumed(
                genealogy,
                unfinished,
                next_action,
            );
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
        progress.say(&Said::Building);
        containers::build_progenitor(&workspace, &launch, time_limit, stop).map_err(|error| {
            match error {
                ContainerError::Stopped => SearchError::Asked,
                error => {
                    progress.say(&Said::BuildFailed {
                        error: &error,
                        workspace: &workspace,
                    });
                    SearchError::Build(error)
                }
            }
        })?;
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
            while !progress.halted(stop) && progress.running() < settings.max_concurrent_containers
            {
                let Some((action, again)) = progress.take() else {
                    break;
                };
                progress.hand_over(action, again, &shared, scope, &sender);
            }
            if progress.running() == 0 {
                break;
            }
            if let Ok(ended) = receiver.recv_timeout(WAIT) {
                progress.end(ended, &shared.workspace);
            }
            if progress.halted(stop) {
                progress.stopping();
                shared.halt.store(true, Ordering::SeqCst);
            }
        }
    });
    progress.finish(&shared.workspace, stop)
}
