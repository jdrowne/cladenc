mod action;
mod error;
mod progress;
mod said;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use crate::containers::{self, ContainerError, Launch};
use crate::genealogy::HarnessId;
use crate::journal::{self, Journal, Record, Settings, State};
use crate::run_directory::RunDirectory;

use action::Ended;
pub use error::DirectedEvolutionError;
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
    launch: Launch<'a>,
    run_directory: RunDirectory,
    halt: AtomicBool,
    sender: Sender<Ended>,
}

pub fn start(
    settings: &Settings,
    path: &Path,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, DirectedEvolutionError> {
    let key = settings.model_key().map_err(DirectedEvolutionError::Key)?;
    let making = Said::Making {
        run_directory: path,
        primogenitor: &settings.primogenitor,
    };
    say(writer, &making);
    let (run_directory, journal) = RunDirectory::create(path, &settings.primogenitor)
        .and_then(|run_directory| {
            settings
                .benchmarks
                .iter()
                .try_for_each(|benchmark| {
                    let copying = Said::CopyingTasks {
                        benchmark: &benchmark.directory,
                        tasks: benchmark.tasks.len() as u64,
                    };
                    say(writer, &copying);
                    run_directory.copy_benchmark(benchmark)
                })
                .map(|()| run_directory)
        })
        .map_err(DirectedEvolutionError::Io)
        .and_then(|run_directory| {
            build_image(settings, &run_directory, stop, writer).map(|()| run_directory)
        })
        .and_then(|run_directory| {
            Journal::open(path)
                .map(|journal| (run_directory, journal))
                .map_err(DirectedEvolutionError::Journal)
        })?;
    let mut progress = Progress::new(settings, journal, writer);
    progress.record(&Record::Run {
        format: journal::FORMAT,
        settings: Box::new(settings.clone()),
    });
    carry_on(settings, key.as_deref(), run_directory, progress, stop)
}

fn build_image(
    settings: &Settings,
    run_directory: &RunDirectory,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<(), DirectedEvolutionError> {
    let given = settings
        .dockerfile
        .as_deref()
        .zip(settings.docker_build_timeout);
    let Some((dockerfile, time_limit)) = given else {
        return Ok(());
    };
    let image = &settings.docker_image;
    say(writer, &Said::BuildingImage { image, dockerfile });
    match containers::build_image(run_directory, dockerfile, image, time_limit, stop) {
        Ok(()) => {
            say(
                writer,
                &Said::ImageBuilt {
                    image,
                    run_directory,
                },
            );
            Ok(())
        }
        Err(error) => {
            say(
                writer,
                &Said::ImageNotBuilt {
                    error: &error,
                    run_directory,
                },
            );
            Err(DirectedEvolutionError::Image(error))
        }
    }
}

pub fn resume(
    path: &Path,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, DirectedEvolutionError> {
    let (journal, state) = Journal::open_existing(path)
        .and_then(|journal| journal::read(path).map(|state| (journal, state)))
        .map_err(DirectedEvolutionError::Unusable)?;
    let State {
        settings,
        genealogy,
        unfinished,
        next_action,
        best_harness,
    } = state;
    let run_directory = RunDirectory::at(path);
    match best_harness {
        Some(best_harness) => {
            let directory = run_directory.harness(best_harness);
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
            let removed = containers::remove_left_behind(&run_directory)
                .map_err(DirectedEvolutionError::Left)?;
            if removed > 0 {
                say(
                    writer,
                    &Said::Removed {
                        containers: removed,
                    },
                );
            }
            let key = settings.model_key().map_err(DirectedEvolutionError::Key)?;
            let continuing = Said::Continuing {
                run_directory: path,
                procreations: genealogy.procreations_started(),
                max_procreations: settings.max_procreations,
                harnesses: genealogy.harness_count(),
            };
            say(writer, &continuing);
            let progress = Progress::new(&settings, journal, writer).resumed(
                genealogy,
                unfinished,
                next_action,
            );
            carry_on(&settings, key.as_deref(), run_directory, progress, stop)
        }
    }
}

fn carry_on(
    settings: &Settings,
    key: Option<&str>,
    run_directory: RunDirectory,
    mut progress: Progress<'_>,
    stop: &AtomicBool,
) -> Result<Found, DirectedEvolutionError> {
    let launch = settings.launch(key);
    if progress.awaits_build(stop) {
        progress.say(&Said::Building);
        containers::build_primogenitor(&run_directory, &launch, stop).map_err(
            |error| match error {
                ContainerError::Stopped => DirectedEvolutionError::Asked,
                error => {
                    progress.say(&Said::BuildFailed {
                        error: &error,
                        run_directory: &run_directory,
                    });
                    DirectedEvolutionError::Build(error)
                }
            },
        )?;
        progress.built();
    }
    let (sender, receiver) = mpsc::channel();
    let shared = Shared {
        launch,
        run_directory,
        halt: AtomicBool::new(false),
        sender,
    };
    thread::scope(|scope| {
        loop {
            while !progress.halted(stop)
                && progress.running() < settings.max_concurrent_containers
                && let Some((action, again)) = progress.take()
            {
                progress.hand_over(action, again, &shared, scope);
            }
            if progress.running() == 0 {
                break;
            }
            if let Ok(ended) = receiver.recv_timeout(WAIT) {
                progress.end(ended, &shared.run_directory);
            }
            if progress.halted(stop) {
                progress.stopping();
                shared.halt.store(true, Ordering::SeqCst);
            }
        }
    });
    progress.finish(&shared.run_directory, stop)
}
