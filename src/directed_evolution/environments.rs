use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::DirectedEvolutionError;
use super::said::{Environments, Said, say};
use crate::benchmark::{self, EnvironmentConfiguration, TaskConfiguration};
use crate::containers::{self, ContainerError, Environment, GraderImage, Machine};
use crate::journal::Settings;
use crate::outputs_directory::OutputsDirectory;

const OF_THE_GRADER: &str = "-grader";
const OF_THE_TASK: [&str; 2] = ["cpus", "memory_mb"];
const OF_ITS_GRADER: [&str; 2] = [
    "verifier.environment.cpus",
    "verifier.environment.memory_mb",
];

pub(super) struct Built {
    images: Vec<String>,
}

impl Built {
    pub(super) fn by(settings: &Settings) -> Self {
        Self {
            images: settings
                .environments()
                .into_iter()
                .flat_map(|(_task, image)| [of_the_grader(&image), image])
                .collect(),
        }
    }
}

impl Drop for Built {
    fn drop(&mut self) {
        containers::remove_images(&self.images);
    }
}

pub(super) fn of_the_grader(image: &str) -> String {
    format!("{image}{OF_THE_GRADER}")
}

#[derive(Clone, Copy)]
pub(super) enum Whose {
    TheTask,
    TheGrader,
}

pub(super) struct Making<'a> {
    pub(super) outputs_directory: &'a OutputsDirectory,
    pub(super) stop: &'a AtomicBool,
    pub(super) writer: &'a mut dyn Write,
    pub(super) machine: Option<Machine>,
}

pub(super) fn made(
    settings: &Settings,
    outputs_directory: &OutputsDirectory,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<BTreeMap<String, Environment>, DirectedEvolutionError> {
    let mut making = Making {
        outputs_directory,
        stop,
        writer,
        machine: None,
    };
    let mut environments = BTreeMap::new();
    let mut count_built = 0;
    for (task, image) in settings.environments() {
        let configuration =
            TaskConfiguration::read(&outputs_directory.task(&task)).map_err(|error| {
                DirectedEvolutionError::Configuration {
                    task: task.clone(),
                    error,
                }
            })?;
        let asked = [
            configuration.cpus,
            megabytes(configuration.memory_megabytes),
        ];
        making.fits(&task, OF_THE_TASK, asked)?;
        let named = of_the_grader(&image);
        let image = match configuration.docker_image.clone() {
            Some(given) => making.take(Whose::TheTask, &task, given)?,
            None => {
                count_built += 1;
                let time_limit = configuration.build_time_limit;
                making.build(Whose::TheTask, &task, image, time_limit)?
            }
        };
        let grader = configuration
            .grader_environment
            .as_ref()
            .map(|own| making.grader(&task, own, [&image, &named]))
            .transpose()?;
        let environment = Environment {
            image,
            configuration,
            grader,
        };
        environments.insert(task, environment);
    }
    if count_built > 0 {
        let built = Said::Environments(Environments::Built {
            built: count_built,
            tasks: environments.len() as u64,
            outputs_directory,
        });
        say(making.writer, &built);
    }
    Ok(environments)
}

pub(super) fn megabytes(memory: Option<u64>) -> Option<f64> {
    memory.map(|megabytes| megabytes as f64)
}

impl Making<'_> {
    fn fits(
        &mut self,
        task: &str,
        keys: [&'static str; 2],
        asked: [Option<f64>; 2],
    ) -> Result<(), DirectedEvolutionError> {
        if asked.iter().all(Option::is_none) {
            return Ok(());
        }
        let machine = match self.machine {
            Some(machine) => machine,
            None => *self
                .machine
                .insert(containers::machine().map_err(DirectedEvolutionError::Machine)?),
        };
        let has = [machine.cpus, machine.memory_megabytes as f64];
        let too_large = keys
            .into_iter()
            .zip(asked)
            .zip(has)
            .find_map(|((key, asked), has)| {
                asked
                    .filter(|asked| *asked > has)
                    .map(|asked| (key, asked.to_string(), has.to_string()))
            });
        match too_large {
            Some((key, asked, has)) => Err(DirectedEvolutionError::TooLarge {
                task: task.to_owned(),
                key,
                asked,
                has,
            }),
            None => Ok(()),
        }
    }

    fn take(
        &mut self,
        whose: Whose,
        task: &str,
        image: String,
    ) -> Result<String, DirectedEvolutionError> {
        let taking = match whose {
            Whose::TheTask => Environments::Taking {
                task,
                image: &image,
            },
            Whose::TheGrader => Environments::GraderTaking {
                task,
                image: &image,
            },
        };
        say(self.writer, &Said::Environments(taking));
        let task = task.to_owned();
        match (containers::find_image(&image), whose) {
            (Ok(()), _) => Ok(image),
            (Err(error), Whose::TheTask) => {
                Err(DirectedEvolutionError::NotOnTheMachine { task, image, error })
            }
            (Err(error), Whose::TheGrader) => {
                Err(DirectedEvolutionError::GraderNotOnTheMachine { task, image, error })
            }
        }
    }

    fn build(
        &mut self,
        whose: Whose,
        task: &str,
        image: String,
        time_limit: Duration,
    ) -> Result<String, DirectedEvolutionError> {
        let building = match whose {
            Whose::TheTask => Environments::Building {
                task,
                image: &image,
            },
            Whose::TheGrader => Environments::GraderBuilding {
                task,
                image: &image,
            },
        };
        say(self.writer, &Said::Environments(building));
        let outputs_directory = self.outputs_directory;
        let built = match whose {
            Whose::TheTask => containers::build_environment(
                outputs_directory,
                task,
                &image,
                time_limit,
                self.stop,
            ),
            Whose::TheGrader => containers::build_grader_environment(
                outputs_directory,
                task,
                &image,
                time_limit,
                self.stop,
            ),
        };
        match built {
            Ok(()) => Ok(image),
            Err(ContainerError::Stopped) => Err(DirectedEvolutionError::Asked),
            Err(error) => Err(self.not_built(whose, task, error)),
        }
    }

    pub(super) fn not_built(
        &mut self,
        whose: Whose,
        task: &str,
        error: ContainerError,
    ) -> DirectedEvolutionError {
        let outputs_directory = self.outputs_directory;
        let not_built = match whose {
            Whose::TheTask => Environments::NotBuilt {
                task,
                error: &error,
                outputs_directory,
            },
            Whose::TheGrader => Environments::GraderNotBuilt {
                task,
                error: &error,
                outputs_directory,
            },
        };
        say(self.writer, &Said::Environments(not_built));
        let task = task.to_owned();
        match whose {
            Whose::TheTask => DirectedEvolutionError::Environment { task, error },
            Whose::TheGrader => DirectedEvolutionError::GraderEnvironment { task, error },
        }
    }

    fn grader(
        &mut self,
        task: &str,
        own: &EnvironmentConfiguration,
        [image, named]: [&str; 2],
    ) -> Result<GraderImage, DirectedEvolutionError> {
        let asked = [own.cpus, megabytes(own.memory_megabytes)];
        self.fits(task, OF_ITS_GRADER, asked)?;
        if let Some(given) = own.docker_image.clone() {
            self.take(Whose::TheGrader, task, given)
                .map(|name| GraderImage {
                    name,
                    holds_grader: true,
                })
        } else if benchmark::holds_grader_dockerfile(&self.outputs_directory.task(task)) {
            self.build(
                Whose::TheGrader,
                task,
                named.to_owned(),
                own.build_time_limit,
            )
            .map(|name| GraderImage {
                name,
                holds_grader: true,
            })
        } else {
            Ok(GraderImage {
                name: image.to_owned(),
                holds_grader: false,
            })
        }
    }
}
