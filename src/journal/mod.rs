mod error;
mod record;
mod state;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub use error::JournalError;
pub use record::{Record, Settings};
pub use state::{Started, State, read};

pub const FORMAT: u32 = 18;

#[derive(Debug)]
pub struct Journal {
    file: File,
}

pub fn path(outputs_directory: &Path) -> PathBuf {
    outputs_directory.join("journal.jsonl")
}

impl Journal {
    pub fn open(outputs_directory: &Path) -> Result<Self, JournalError> {
        Self::locked(OpenOptions::new().create(true), outputs_directory)
    }

    pub fn open_existing(outputs_directory: &Path) -> Result<Self, JournalError> {
        Self::locked(&mut OpenOptions::new(), outputs_directory)
    }

    fn locked(options: &mut OpenOptions, outputs_directory: &Path) -> Result<Self, JournalError> {
        let mut file = options
            .read(true)
            .append(true)
            .open(path(outputs_directory))
            .map_err(JournalError::Io)?;
        file.try_lock().map_err(|_| JournalError::Locked)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .and_then(|_| file.set_len(whole_lines(&contents).len() as u64))
            .map(|()| Self { file })
            .map_err(JournalError::Io)
    }

    pub fn append(&mut self, record: &Record) -> Result<(), JournalError> {
        let line = record.line()? + "\n";
        self.file
            .write_all(line.as_bytes())
            .and_then(|()| self.file.sync_all())
            .map_err(JournalError::Io)
    }
}

impl Drop for Journal {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn whole_lines(contents: &[u8]) -> &[u8] {
    let end = contents
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |last| last + 1);
    &contents[..end]
}
