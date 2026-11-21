use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::outputs_directory::holds_only_harness_files;

pub(crate) const SPECIFICATION: &str = "instruction.md";
const WORKSPACE: &str = "workspace";
pub(crate) const TESTS: &str = "tests";
const GRADER: &str = "test.sh";
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
    pub fn read(directory: &Path) -> Result<Self, BenchmarkError> {
        let benchmark = Self {
            directory: directory.to_owned(),
            tasks: tasks(directory)?,
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
    NotNamed { directory: String },
    GivenTwice { directory: PathBuf },
    SameName { name: String },
    NotAnId { directory: String },
    NoSpecification { task: String },
    NoWorkspace { task: String },
    NoGrader { task: String },
    NoEnvironment { task: String },
    ManyContainers { task: String },
    OtherFiles { task: String },
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
            Self::NoWorkspace { task } => {
                write!(
                    formatter,
                    "task {task} holds no directory named {WORKSPACE}"
                )
            }
            Self::NoGrader { task } => write!(
                formatter,
                "task {task} holds no directory named {TESTS} with an executable file named \
                 {GRADER} in it"
            ),
            Self::NoEnvironment { task } => write!(
                formatter,
                "task {task} holds no directory named {ENVIRONMENT} with a file named \
                 {DOCKERFILE} in it"
            ),
            Self::ManyContainers { task } => write!(
                formatter,
                "task {task} holds a Compose file in its directory named {ENVIRONMENT}, and an \
                 environment of more than one container cannot be built"
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

fn tasks(benchmark: &Path) -> Result<Vec<String>, BenchmarkError> {
    let mut directories = fs::read_dir(benchmark)
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<io::Result<Vec<PathBuf>>>()
        })
        .map_err(|_| BenchmarkError::NotADirectory)?;
    directories.retain(|entry| entry.is_dir());
    directories.sort();
    if is_specification(&benchmark.join(SPECIFICATION))
        && is_directory(&benchmark.join(WORKSPACE))
        && is_grader(benchmark)
    {
        return Err(BenchmarkError::OneTask);
    }
    if directories.is_empty() {
        return Err(BenchmarkError::NoTask);
    }
    directories
        .iter()
        .map(|directory| task(directory))
        .collect()
}

fn name_of(directory: &Path) -> String {
    directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn task(directory: &Path) -> Result<String, BenchmarkError> {
    let task = name_of(directory);
    if !is_name(&task) {
        return Err(BenchmarkError::NotAnId { directory: task });
    }
    if !is_specification(&directory.join(SPECIFICATION)) {
        return Err(BenchmarkError::NoSpecification { task });
    }
    if !is_directory(&directory.join(WORKSPACE)) {
        return Err(BenchmarkError::NoWorkspace { task });
    }
    if !is_grader(directory) {
        return Err(BenchmarkError::NoGrader { task });
    }
    if !is_environment(directory) {
        return Err(BenchmarkError::NoEnvironment { task });
    }
    if holds_compose_file(directory) {
        return Err(BenchmarkError::ManyContainers { task });
    }
    if !can_be_copied(directory) {
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

fn is_specification(file: &Path) -> bool {
    fs::symlink_metadata(file).is_ok_and(|specification| specification.is_file())
        && fs::read_to_string(file)
            .is_ok_and(|text| !text.trim().is_empty() && !text.contains('\0'))
}

fn is_directory(directory: &Path) -> bool {
    fs::symlink_metadata(directory).is_ok_and(|files| files.is_dir())
}

fn is_grader(task: &Path) -> bool {
    let tests = task.join(TESTS);
    is_directory(&tests) && is_program(&tests.join(GRADER))
}

fn is_environment(task: &Path) -> bool {
    let environment = task.join(ENVIRONMENT);
    is_directory(&environment)
        && fs::symlink_metadata(environment.join(DOCKERFILE))
            .is_ok_and(|dockerfile| dockerfile.is_file())
}

fn holds_compose_file(task: &Path) -> bool {
    COMPOSE_FILES
        .iter()
        .any(|file| fs::symlink_metadata(task.join(ENVIRONMENT).join(file)).is_ok())
}

fn is_program(file: &Path) -> bool {
    fs::symlink_metadata(file)
        .is_ok_and(|grader| grader.is_file() && grader.permissions().mode() & 0o111 != 0)
}

fn can_be_copied(task: &Path) -> bool {
    fs::canonicalize(task)
        .and_then(|resolved| holds_only_harness_files(&resolved))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests;
