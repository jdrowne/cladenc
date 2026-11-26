mod error;
mod record;
mod state;

use std::path::{Path, PathBuf};

use crate::file_system::{FileSystem, Opened};

pub use error::JournalError;
pub(crate) use record::Record;
pub use record::Settings;
pub use state::Started;
pub(crate) use state::{Seconds, State, read};

pub(crate) const FORMAT: u32 = 36;

#[derive(Debug)]
pub(crate) struct Journal {
    file: Box<dyn Opened>,
}

pub fn path(outputs_directory: &Path) -> PathBuf {
    outputs_directory.join("journal.jsonl")
}

impl Journal {
    pub(crate) fn open(
        files: &dyn FileSystem,
        outputs_directory: &Path,
    ) -> Result<Self, JournalError> {
        Self::locked(files, outputs_directory, true)
    }

    pub(crate) fn open_existing(
        files: &dyn FileSystem,
        outputs_directory: &Path,
    ) -> Result<Self, JournalError> {
        Self::locked(files, outputs_directory, false)
    }

    fn locked(
        files: &dyn FileSystem,
        outputs_directory: &Path,
        make: bool,
    ) -> Result<Self, JournalError> {
        let mut file = files
            .open(&path(outputs_directory), make)
            .map_err(JournalError::Io)?;
        if !file.lock() {
            return Err(JournalError::Locked);
        }
        file.contents()
            .and_then(|contents| file.cut(whole_lines(&contents).len() as u64))
            .map(|()| Self { file })
            .map_err(JournalError::Io)
    }

    pub(crate) fn append(&mut self, record: &Record) -> Result<(), JournalError> {
        let line = record.line()? + "\n";
        self.file.append(line.as_bytes()).map_err(JournalError::Io)
    }
}

fn whole_lines(contents: &[u8]) -> &[u8] {
    let end = contents
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |last| last + 1);
    &contents[..end]
}

#[cfg(test)]
mod tests;
