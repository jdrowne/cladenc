mod artifacts;
mod configuration;
mod valid;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::file_system::{FileSystem, Kind};
use crate::outputs_directory::holds_only_harness_files;

pub(crate) use artifacts::Artifact;
use configuration::CONFIGURATION;
pub use configuration::{
    CollectCommand, ConfigurationError, EnvironmentConfiguration, HealthCheck, TaskConfiguration,
};

pub(crate) const SPECIFICATION: &str = "instruction.md";
pub(crate) const TESTS: &str = "tests";
const VERIFIER: &str = "test.sh";
pub(crate) const ENVIRONMENT: &str = "environment";
const DOCKERFILE: &str = "Dockerfile";
const COMPOSE_FILES: [&str; 4] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Benchmark {
    pub directory: PathBuf,
    pub tasks: Vec<String>,
}

impl Benchmark {
    pub fn read(files: &dyn FileSystem, directory: &Path) -> Result<Self, BenchmarkError> {
        let benchmark = Self {
            directory: directory.to_owned(),
            tasks: tasks(files, directory)?,
        };
        if is_name(&benchmark.name()) {
            Ok(benchmark)
        } else {
            Err(BenchmarkError::NotNamed {
                directory: benchmark.name(),
            })
        }
    }

    pub(crate) fn name(&self) -> String {
        name_of(&self.directory)
    }

    pub(crate) fn task(&self, id: &str) -> String {
        format!("{}/{id}", self.name())
    }
}

pub(crate) fn told_apart(benchmarks: &[Benchmark]) -> Result<(), BenchmarkError> {
    benchmarks
        .iter()
        .enumerate()
        .try_for_each(|(place, benchmark)| {
            benchmarks[..place]
                .iter()
                .find(|earlier| earlier.name() == benchmark.name())
                .map_or(Ok(()), |earlier| Err(not_told_apart(earlier, benchmark)))
        })
}

fn not_told_apart(earlier: &Benchmark, benchmark: &Benchmark) -> BenchmarkError {
    if earlier.directory == benchmark.directory {
        BenchmarkError::GivenTwice {
            directory: benchmark.directory.clone(),
        }
    } else {
        BenchmarkError::SameName {
            name: benchmark.name(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkError {
    NotADirectory,
    OneTask,
    NoTask,
    NotNamed {
        directory: String,
    },
    GivenTwice {
        directory: PathBuf,
    },
    SameName {
        name: String,
    },
    NotAnId {
        directory: String,
    },
    NoSpecification {
        task: String,
    },
    NoVerifier {
        task: String,
    },
    NoConfiguration {
        task: String,
    },
    Configuration {
        task: String,
        error: ConfigurationError,
    },
    NoEnvironment {
        task: String,
    },
    ManyContainers {
        task: String,
    },
    ManyVerifierContainers {
        task: String,
    },
    OtherFiles {
        task: String,
    },
}

impl Display for BenchmarkError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory => write!(
                formatter,
                "what was given is not a directory that can be read"
            ),
            Self::OneTask => write!(
                formatter,
                "what was given is the directory of one task, and a benchmark is the directory \
                 that holds the tasks"
            ),
            Self::NoTask => write!(
                formatter,
                "the directory given holds no directory, and so no task"
            ),
            Self::NotNamed { directory } => write!(
                formatter,
                "its directory is named `{directory}`, and the name of a benchmark is made of \
                 letters, digits, dots, underscores and hyphens alone"
            ),
            Self::GivenTwice { directory } => write!(
                formatter,
                "the benchmark {} is given twice",
                directory.display()
            ),
            Self::SameName { name } => write!(
                formatter,
                "two of the benchmarks given are in directories named `{name}`, and a task is \
                 known by the name of its benchmark and its id together"
            ),
            Self::NotAnId { directory } => write!(
                formatter,
                "the directory `{directory}` in it is not named with letters, digits, dots, \
                 underscores and hyphens alone, as a task is"
            ),
            Self::NoSpecification { task } => write!(
                formatter,
                "task {task} holds no file of text named {SPECIFICATION}"
            ),
            Self::NoVerifier { task } => write!(
                formatter,
                "task {task} holds no directory named {TESTS} with an executable file named \
                 {VERIFIER} in it"
            ),
            Self::NoConfiguration { task } => {
                write!(formatter, "task {task} holds no file named {CONFIGURATION}")
            }
            Self::Configuration { task, error } => {
                write!(
                    formatter,
                    "task {task} holds a {CONFIGURATION} that {error}"
                )
            }
            Self::NoEnvironment { task } => write!(
                formatter,
                "task {task} holds no directory named {ENVIRONMENT} with a file named \
                 {DOCKERFILE} in it, and its {CONFIGURATION} names no image in its place"
            ),
            Self::ManyContainers { task } => write!(
                formatter,
                "task {task} holds a Compose file in its directory named {ENVIRONMENT}, and an \
                 environment of more than one container cannot be built"
            ),
            Self::ManyVerifierContainers { task } => write!(
                formatter,
                "task {task} has its verifier run in a container of its own and holds a Compose \
                 file in its directory named {TESTS}, and an environment of more than one \
                 container cannot be built"
            ),
            Self::OtherFiles { task } => write!(
                formatter,
                "task {task} holds something that is not a regular file, a directory or a \
                 symbolic link"
            ),
        }
    }
}

impl Error for BenchmarkError {}

fn tasks(files: &dyn FileSystem, benchmark: &Path) -> Result<Vec<String>, BenchmarkError> {
    let mut directories: Vec<PathBuf> = files
        .list(benchmark)
        .map_err(|_| BenchmarkError::NotADirectory)?
        .iter()
        .map(|name| benchmark.join(name))
        .filter(|entry| leads_to_a_directory(files, entry))
        .collect();
    directories.sort();
    if is_specification(files, &benchmark.join(SPECIFICATION)) && is_verifier(files, benchmark) {
        return Err(BenchmarkError::OneTask);
    }
    if directories.is_empty() {
        return Err(BenchmarkError::NoTask);
    }
    directories
        .iter()
        .map(|directory| task(files, directory))
        .collect()
}

fn leads_to_a_directory(files: &dyn FileSystem, path: &Path) -> bool {
    files
        .followed(path)
        .is_ok_and(|found| found.kind == Kind::Directory)
}

fn name_of(directory: &Path) -> String {
    directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn task(files: &dyn FileSystem, directory: &Path) -> Result<String, BenchmarkError> {
    let task = name_of(directory);
    if !is_name(&task) {
        return Err(BenchmarkError::NotAnId { directory: task });
    }
    if !is_specification(files, &directory.join(SPECIFICATION)) {
        return Err(BenchmarkError::NoSpecification { task });
    }
    let configuration = TaskConfiguration::read(files, directory);
    let in_an_image = configuration
        .as_ref()
        .is_ok_and(|configuration| verifier_is_in_an_image(files, configuration, directory));
    if !in_an_image && !is_verifier(files, directory) {
        return Err(BenchmarkError::NoVerifier { task });
    }
    let configuration = match configuration {
        Ok(configuration) => configuration,
        Err(ConfigurationError::Missing) => return Err(BenchmarkError::NoConfiguration { task }),
        Err(error) => return Err(BenchmarkError::Configuration { task, error }),
    };
    if configuration.docker_image.is_none() && !is_environment(files, directory) {
        return Err(BenchmarkError::NoEnvironment { task });
    }
    if holds_compose_file(files, &directory.join(ENVIRONMENT)) {
        return Err(BenchmarkError::ManyContainers { task });
    }
    if configuration.verifier_environment.is_some()
        && holds_compose_file(files, &directory.join(TESTS))
    {
        return Err(BenchmarkError::ManyVerifierContainers { task });
    }
    if !can_be_copied(files, directory) {
        return Err(BenchmarkError::OtherFiles { task });
    }
    Ok(task)
}

fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
}

fn is_specification(files: &dyn FileSystem, file: &Path) -> bool {
    is_a(files, file, Kind::File)
        && files
            .read_text(file)
            .is_ok_and(|text| !text.trim().is_empty() && !text.contains('\0'))
}

fn is_a(files: &dyn FileSystem, path: &Path, kind: Kind) -> bool {
    files.found(path).is_ok_and(|found| found.kind == kind)
}

fn is_verifier(files: &dyn FileSystem, task: &Path) -> bool {
    let tests = task.join(TESTS);
    is_a(files, &tests, Kind::Directory) && is_program(files, &tests.join(VERIFIER))
}

fn is_environment(files: &dyn FileSystem, task: &Path) -> bool {
    holds_dockerfile(files, &task.join(ENVIRONMENT))
}

pub(crate) fn holds_verifier_dockerfile(files: &dyn FileSystem, task: &Path) -> bool {
    holds_dockerfile(files, &task.join(TESTS))
}

fn holds_dockerfile(files: &dyn FileSystem, directory: &Path) -> bool {
    is_a(files, directory, Kind::Directory) && is_a(files, &directory.join(DOCKERFILE), Kind::File)
}

fn verifier_is_in_an_image(
    files: &dyn FileSystem,
    configuration: &TaskConfiguration,
    task: &Path,
) -> bool {
    configuration
        .verifier_environment
        .as_ref()
        .is_some_and(|own| own.docker_image.is_some() || holds_verifier_dockerfile(files, task))
}

fn holds_compose_file(files: &dyn FileSystem, directory: &Path) -> bool {
    COMPOSE_FILES
        .iter()
        .any(|file| files.found(&directory.join(file)).is_ok())
}

fn is_program(files: &dyn FileSystem, file: &Path) -> bool {
    files
        .found(file)
        .is_ok_and(|verifier| verifier.kind == Kind::File && verifier.mode & 0o111 != 0)
}

fn can_be_copied(files: &dyn FileSystem, task: &Path) -> bool {
    files
        .resolved(task)
        .and_then(|resolved| holds_only_harness_files(files, &resolved))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests;
