mod record;
mod state;

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::genealogy::{GenealogyError, HarnessId};

pub use record::{Record, Settings};
pub use state::{Started, State, read};

pub const FORMAT: u32 = 1;

#[derive(Debug)]
pub enum JournalError {
    Io(io::Error),
    Locked,
    NotARecord { line: usize },
    FirstRecordNotRun,
    UnknownFormat { format: u32 },
    RunRecordRepeated { line: usize },
    Refused { line: usize, error: GenealogyError },
    WrongChild { line: usize, child: HarnessId },
    AfterSearchFinished { line: usize },
}

#[derive(Debug)]
pub struct Journal {
    file: File,
}

pub fn path(run_directory: &Path) -> PathBuf {
    run_directory.join("journal.jsonl")
}

impl Journal {
    pub fn open(run_directory: &Path) -> Result<Self, JournalError> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path(run_directory))
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

fn whole_lines(contents: &[u8]) -> &[u8] {
    let end = contents
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |last| last + 1);
    &contents[..end]
}
