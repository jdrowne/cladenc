use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::workspace::holds_only_harness_files;

pub const STATEMENT: &str = "statement";
pub const FILES: &str = "files";
pub const CHECK: &str = "check";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkError {
    NotADirectory,
    OneTask,
    NoTask,
    NotAnId { directory: String },
    NoStatement { task: String },
    NoFiles { task: String },
    NoCheck { task: String },
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
            Self::NotAnId { directory } => write!(
                formatter,
                "the directory `{directory}` in it is not named with letters, digits, dots, \
                 underscores and hyphens alone, as a task is"
            ),
            Self::NoStatement { task } => write!(
                formatter,
                "task {task} holds no file of text named {STATEMENT}"
            ),
            Self::NoFiles { task } => {
                write!(formatter, "task {task} holds no directory named {FILES}")
            }
            Self::NoCheck { task } => write!(
                formatter,
                "task {task} holds no executable file named {CHECK}"
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

pub fn tasks(benchmark: &Path) -> Result<Vec<String>, BenchmarkError> {
    let mut directories = fs::read_dir(benchmark)
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<io::Result<Vec<PathBuf>>>()
        })
        .map_err(|_| BenchmarkError::NotADirectory)?;
    directories.retain(|entry| entry.is_dir());
    directories.sort();
    if is_statement(&benchmark.join(STATEMENT))
        && is_files(&benchmark.join(FILES))
        && is_program(&benchmark.join(CHECK))
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

fn task(directory: &Path) -> Result<String, BenchmarkError> {
    let task = directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !is_task_id(&task) {
        return Err(BenchmarkError::NotAnId { directory: task });
    }
    if !is_statement(&directory.join(STATEMENT)) {
        return Err(BenchmarkError::NoStatement { task });
    }
    if !is_files(&directory.join(FILES)) {
        return Err(BenchmarkError::NoFiles { task });
    }
    if !is_program(&directory.join(CHECK)) {
        return Err(BenchmarkError::NoCheck { task });
    }
    if !can_be_copied(directory) {
        return Err(BenchmarkError::OtherFiles { task });
    }
    Ok(task)
}

fn is_task_id(name: &str) -> bool {
    name.chars()
        .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
}

fn is_statement(file: &Path) -> bool {
    fs::symlink_metadata(file).is_ok_and(|statement| statement.is_file())
        && fs::read_to_string(file)
            .is_ok_and(|text| !text.trim().is_empty() && !text.contains('\0'))
}

fn is_files(directory: &Path) -> bool {
    fs::symlink_metadata(directory).is_ok_and(|files| files.is_dir())
}

fn is_program(file: &Path) -> bool {
    fs::symlink_metadata(file)
        .is_ok_and(|check| check.is_file() && check.permissions().mode() & 0o111 != 0)
}

fn can_be_copied(task: &Path) -> bool {
    fs::canonicalize(task)
        .and_then(|resolved| holds_only_harness_files(&resolved))
        .unwrap_or(false)
}
