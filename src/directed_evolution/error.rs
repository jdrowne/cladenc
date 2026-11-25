use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;

use crate::benchmark::ConfigurationError;
use crate::containers::ContainerError;
use crate::journal::{JournalError, Started};

#[derive(Debug)]
pub enum DirectedEvolutionError {
    Io(io::Error),
    Unusable(JournalError),
    Key(PathBuf),
    Left(io::Error),
    Image(ContainerError),
    Configuration {
        task: String,
        error: ConfigurationError,
    },
    NotOnTheMachine {
        task: String,
        image: String,
        error: io::Error,
    },
    Machine(io::Error),
    TooLarge {
        task: String,
        key: &'static str,
        asked: String,
        has: String,
    },
    Environment {
        task: String,
        error: ContainerError,
    },
    VerifierNotOnTheMachine {
        task: String,
        image: String,
        error: io::Error,
    },
    VerifierEnvironment {
        task: String,
        error: ContainerError,
    },
    Journal(JournalError),
    NotStarted,
    Asked,
    Build(ContainerError),
    Action {
        action: Started,
        error: ContainerError,
    },
}

impl Display for DirectedEvolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "the outputs directory cannot be made: {error}"),
            Self::Unusable(error) => {
                write!(formatter, "the outputs directory cannot be used: {error}")
            }
            Self::Key(file) => write!(
                formatter,
                "the key of the model cannot be read: {} is not a file that holds a key on one \
                 line and nothing else",
                file.display()
            ),
            Self::Left(error) => write!(
                formatter,
                "the containers that the directed evolution left running cannot be removed: {error}"
            ),
            Self::Image(error) => write!(
                formatter,
                "the image of the directed evolution was not built: its build {error}"
            ),
            Self::Configuration { task, error } => write!(
                formatter,
                "task {task} cannot be attempted: its task.toml {error}"
            ),
            Self::NotOnTheMachine { task, image, error } => write!(
                formatter,
                "the environment of task {task} is not on the machine: its task.toml names the \
                 image {image}, and cladenc fetches no image: {error}"
            ),
            Self::Machine(error) => {
                write!(formatter, "what the machine has cannot be told: {error}")
            }
            Self::TooLarge {
                task,
                key,
                asked,
                has,
            } => write!(
                formatter,
                "task {task} asks for more than the machine has: {asked} under {key} in its \
                 task.toml, where Docker has {has}"
            ),
            Self::Environment { task, error } => write!(
                formatter,
                "the environment of task {task} was not built: its build {error}"
            ),
            Self::VerifierNotOnTheMachine { task, image, error } => write!(
                formatter,
                "the verifier environment of task {task} is not on the machine: its \
                 task.toml names the image {image}, and cladenc fetches no image: {error}"
            ),
            Self::VerifierEnvironment { task, error } => write!(
                formatter,
                "the verifier environment of task {task} was not built: its build {error}"
            ),
            Self::Journal(error) => write!(
                formatter,
                "the directed evolution stopped before its end: its journal could not be \
                 written: {error}"
            ),
            Self::NotStarted => write!(
                formatter,
                "the directed evolution was stopped before it was started, so `cladenc resume` \
                 has nothing to continue"
            ),
            Self::Asked => write!(
                formatter,
                "the directed evolution stopped before its end: it was asked to stop"
            ),
            Self::Build(error) => write!(
                formatter,
                "the directed evolution stopped before its end: the build of the primogenitor {error}"
            ),
            Self::Action { action, error } => {
                write!(
                    formatter,
                    "the directed evolution stopped before its end: {action} {error}"
                )
            }
        }
    }
}

impl Error for DirectedEvolutionError {}
