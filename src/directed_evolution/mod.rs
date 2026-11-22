mod action;
mod error;
mod progress;
mod said;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use crate::benchmark::TaskConfiguration;
use crate::containers::{self, ContainerError, Environment, Launch, Machine};
use crate::genealogy::HarnessId;
use crate::holdout::TaskSet;
use crate::journal::{self, Journal, Record, Settings, State};
use crate::outputs_directory::OutputsDirectory;

use action::Ended;
pub use error::DirectedEvolutionError;
use progress::Progress;
use said::{Environments, Said, say};

const WAIT: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Found {
    pub best_harness: HarnessId,
    pub directory: PathBuf,
}

struct Shared<'a> {
    launch: Launch<'a>,
    environments: BTreeMap<String, Environment>,
    outputs_directory: OutputsDirectory,
    halt: AtomicBool,
    sender: Sender<Ended>,
}

struct Made {
    outputs_directory: OutputsDirectory,
    environments: BTreeMap<String, Environment>,
    built: Built,
}

struct Built {
    images: Vec<String>,
}

impl Built {
    fn by(settings: &Settings) -> Self {
        Self {
            images: settings
                .environments()
                .into_iter()
                .map(|(_task, image)| image)
                .collect(),
        }
    }
}

impl Drop for Built {
    fn drop(&mut self) {
        containers::remove_images(&self.images);
    }
}

fn not_started(error: DirectedEvolutionError) -> DirectedEvolutionError {
    match error {
        DirectedEvolutionError::Asked | DirectedEvolutionError::Image(ContainerError::Stopped) => {
            DirectedEvolutionError::NotStarted
        }
        other => other,
    }
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
    let (made, journal) = OutputsDirectory::create(path, &settings.primogenitor)
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
            let built = Built::by(settings);
            build_image(settings, &outputs_directory, stop, writer)
                .and_then(|()| environments(settings, &outputs_directory, stop, writer))
                .map(|environments| Made {
                    outputs_directory,
                    environments,
                    built,
                })
        })
        .map_err(not_started)
        .and_then(|made| {
            Journal::open(path)
                .map(|journal| (made, journal))
                .map_err(DirectedEvolutionError::Journal)
        })?;
    let mut progress = Progress::new(settings, journal, writer);
    progress.record(&Record::Run {
        format: journal::FORMAT,
        settings: Box::new(settings.clone()),
    });
    carry_on(settings, key.as_deref(), made, progress, stop)
}

fn say_the_holdout(settings: &Settings, writer: &mut dyn Write) {
    for benchmark in &settings.benchmarks {
        let holding_out = Said::HoldingOut {
            benchmark: &benchmark.directory,
            selection: settings.count_of(benchmark, TaskSet::Selection),
            corroboration: settings.count_of(benchmark, TaskSet::Corroboration),
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

fn environments(
    settings: &Settings,
    outputs_directory: &OutputsDirectory,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<BTreeMap<String, Environment>, DirectedEvolutionError> {
    let mut environments = BTreeMap::new();
    let mut count_built = 0;
    let mut machine = None;
    for (task, image) in settings.environments() {
        let configuration =
            TaskConfiguration::read(&outputs_directory.task(&task)).map_err(|error| {
                DirectedEvolutionError::Configuration {
                    task: task.clone(),
                    error,
                }
            })?;
        fits(&task, &configuration, &mut machine)?;
        let image = match configuration.docker_image.clone() {
            Some(named) => take_image(&task, named, writer)?,
            None => {
                count_built += 1;
                let building = Building {
                    task: &task,
                    image,
                    time_limit: configuration.build_time_limit,
                };
                build_environment(outputs_directory, building, stop, writer)?
            }
        };
        let environment = Environment {
            image,
            configuration,
        };
        environments.insert(task, environment);
    }
    if count_built > 0 {
        let built = Said::Environments(Environments::Built {
            built: count_built,
            tasks: environments.len() as u64,
            outputs_directory,
        });
        say(writer, &built);
    }
    Ok(environments)
}

fn fits(
    task: &str,
    configuration: &TaskConfiguration,
    machine: &mut Option<Machine>,
) -> Result<(), DirectedEvolutionError> {
    if configuration.cpus.is_none() && configuration.memory_megabytes.is_none() {
        return Ok(());
    }
    let machine = match machine {
        Some(machine) => *machine,
        None => *machine.insert(containers::machine().map_err(DirectedEvolutionError::Machine)?),
    };
    let cpus = configuration
        .cpus
        .filter(|cpus| *cpus > machine.cpus)
        .map(|cpus| ("cpus", cpus.to_string(), machine.cpus.to_string()));
    let memory = configuration
        .memory_megabytes
        .filter(|megabytes| *megabytes > machine.memory_megabytes)
        .map(|megabytes| {
            (
                "memory_mb",
                megabytes.to_string(),
                machine.memory_megabytes.to_string(),
            )
        });
    match cpus.or(memory) {
        Some((key, asked, has)) => Err(DirectedEvolutionError::TooLarge {
            task: task.to_owned(),
            key,
            asked,
            has,
        }),
        None => Ok(()),
    }
}

fn take_image(
    task: &str,
    image: String,
    writer: &mut dyn Write,
) -> Result<String, DirectedEvolutionError> {
    let taking = Said::Environments(Environments::Taking {
        task,
        image: &image,
    });
    say(writer, &taking);
    match containers::find_image(&image) {
        Ok(()) => Ok(image),
        Err(error) => Err(DirectedEvolutionError::NotOnTheMachine {
            task: task.to_owned(),
            image,
            error,
        }),
    }
}

struct Building<'a> {
    task: &'a str,
    image: String,
    time_limit: Duration,
}

fn build_environment(
    outputs_directory: &OutputsDirectory,
    building: Building<'_>,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<String, DirectedEvolutionError> {
    let Building {
        task,
        image,
        time_limit,
    } = building;
    let building = Said::Environments(Environments::Building {
        task,
        image: &image,
    });
    say(writer, &building);
    match containers::build_environment(outputs_directory, task, &image, time_limit, stop) {
        Ok(()) => Ok(image),
        Err(ContainerError::Stopped) => Err(DirectedEvolutionError::Asked),
        Err(error) => {
            let not_built = Said::Environments(Environments::NotBuilt {
                task,
                error: &error,
                outputs_directory,
            });
            say(writer, &not_built);
            Err(DirectedEvolutionError::Environment {
                task: task.to_owned(),
                error,
            })
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
        ..
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
            let built = Built::by(&settings);
            let environments = environments(&settings, &outputs_directory, stop, writer)?;
            let made = Made {
                outputs_directory,
                environments,
                built,
            };
            let progress = Progress::new(&settings, journal, writer).resumed(
                genealogy,
                unfinished,
                next_action,
            );
            carry_on(&settings, key.as_deref(), made, progress, stop)
        }
    }
}

fn carry_on(
    settings: &Settings,
    key: Option<&str>,
    made: Made,
    mut progress: Progress<'_>,
    stop: &AtomicBool,
) -> Result<Found, DirectedEvolutionError> {
    let Made {
        outputs_directory,
        environments,
        built: _built,
    } = made;
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
        environments,
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

#[cfg(test)]
mod tests;
