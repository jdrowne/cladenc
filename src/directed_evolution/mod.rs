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
use crate::outputs_directory::OutputsDirectory;

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
    outputs_directory: OutputsDirectory,
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
        outputs_directory: path,
        primogenitor: &settings.primogenitor,
    };
    say(writer, &making);
    let (outputs_directory, journal) = OutputsDirectory::create(path, &settings.primogenitor)
        .and_then(|outputs_directory| {
            settings
                .benchmarks
                .iter()
                .try_for_each(|benchmark| {
                    let copying = Said::CopyingTasks {
                        benchmark: &benchmark.directory,
                        tasks: benchmark.tasks.len() as u64,
                    };
                    say(writer, &copying);
                    outputs_directory.copy_benchmark(benchmark)
                })
                .map(|()| outputs_directory.holding_out(&settings.holdout_tasks))
        })
        .map_err(DirectedEvolutionError::Io)
        .inspect(|_| say_the_holdout(settings, writer))
        .and_then(|outputs_directory| {
            build_image(settings, &outputs_directory, stop, writer).map(|()| outputs_directory)
        })
        .and_then(|outputs_directory| {
            Journal::open(path)
                .map(|journal| (outputs_directory, journal))
                .map_err(DirectedEvolutionError::Journal)
        })?;
    let mut progress = Progress::new(settings, journal, writer);
    progress.record(&Record::Run {
        format: journal::FORMAT,
        settings: Box::new(settings.clone()),
    });
    carry_on(settings, key.as_deref(), outputs_directory, progress, stop)
}

fn say_the_holdout(settings: &Settings, writer: &mut dyn Write) {
    for benchmark in &settings.benchmarks {
        let holding_out = Said::HoldingOut {
            benchmark: &benchmark.directory,
            held_out: settings.held_out_of(benchmark),
            tasks: benchmark.tasks.len() as u64,
        };
        say(writer, &holding_out);
    }
}

fn build_image(
    settings: &Settings,
    outputs_directory: &OutputsDirectory,
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
    match containers::build_image(outputs_directory, dockerfile, image, time_limit, stop) {
        Ok(()) => {
            say(
                writer,
                &Said::ImageBuilt {
                    image,
                    outputs_directory,
                },
            );
            Ok(())
        }
        Err(error) => {
            say(
                writer,
                &Said::ImageNotBuilt {
                    error: &error,
                    outputs_directory,
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
    let outputs_directory = OutputsDirectory::at(path).holding_out(&settings.holdout_tasks);
    match best_harness {
        Some(best_harness) => {
            let directory = outputs_directory.harness(best_harness);
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
            let removed = containers::remove_left_behind(&outputs_directory)
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
                outputs_directory: path,
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
            carry_on(&settings, key.as_deref(), outputs_directory, progress, stop)
        }
    }
}

fn carry_on(
    settings: &Settings,
    key: Option<&str>,
    outputs_directory: OutputsDirectory,
    mut progress: Progress<'_>,
    stop: &AtomicBool,
) -> Result<Found, DirectedEvolutionError> {
    let launch = settings.launch(key);
    if progress.awaits_build(stop) {
        progress.say(&Said::Building);
        containers::build_primogenitor(&outputs_directory, &launch, stop).map_err(|error| {
            match error {
                ContainerError::Stopped => DirectedEvolutionError::Asked,
                error => {
                    progress.say(&Said::BuildFailed {
                        error: &error,
                        outputs_directory: &outputs_directory,
                    });
                    DirectedEvolutionError::Build(error)
                }
            }
        })?;
        progress.built();
    }
    let (sender, receiver) = mpsc::channel();
    let shared = Shared {
        launch,
        outputs_directory,
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
                progress.end(ended, &shared.outputs_directory);
            }
            if progress.halted(stop) {
                progress.stopping();
                shared.halt.store(true, Ordering::SeqCst);
            }
        }
    });
    progress.finish(&shared.outputs_directory, stop)
}
