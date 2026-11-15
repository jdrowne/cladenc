use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;

use crate::job::JobError;
use crate::journal::{JournalError, Started};

#[derive(Debug)]
pub enum SearchError {
    Io(io::Error),
    Unusable(JournalError),
    Key(PathBuf),
    Journal(JournalError),
    Asked,
    Build(JobError),
    Job { job: Started, error: JobError },
}

impl Display for SearchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "the run directory cannot be made: {error}"),
            Self::Unusable(error) => {
                write!(formatter, "the run directory cannot be used: {error}")
            }
            Self::Key(file) => write!(
                formatter,
                "the key of the model cannot be read: {} is not a file that holds a key on one \
                 line and nothing else",
                file.display()
            ),
            Self::Journal(error) => write!(
                formatter,
                "the search stopped before its end: its journal could not be written: {error}"
            ),
            Self::Asked => write!(
                formatter,
                "the search stopped before its end: it was asked to stop"
            ),
            Self::Build(error) => write!(
                formatter,
                "the search stopped before its end: the build of the progenitor {error}"
            ),
            Self::Job { job, error } => {
                write!(
                    formatter,
                    "the search stopped before its end: {job} {error}"
                )
            }
        }
    }
}

impl Error for SearchError {}
