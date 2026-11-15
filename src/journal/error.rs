use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use crate::genealogy::{GenealogyError, HarnessId};

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

impl Display for JournalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Locked => write!(formatter, "another cladenc is using it"),
            Self::NotARecord { line } => {
                write!(formatter, "line {line} of its journal is not a record")
            }
            Self::FirstRecordNotRun => {
                write!(formatter, "its journal does not start with a run record")
            }
            Self::UnknownFormat { format } => write!(
                formatter,
                "its journal has format {format}, which this cladenc does not read"
            ),
            Self::RunRecordRepeated { line } => {
                write!(
                    formatter,
                    "line {line} of its journal is a second run record"
                )
            }
            Self::Refused { line, error } => write!(
                formatter,
                "line {line} of its journal records what cannot have happened: {error}"
            ),
            Self::WrongChild { line, child } => write!(
                formatter,
                "line {line} of its journal starts harness {child} out of order"
            ),
            Self::AfterSearchFinished { line } => write!(
                formatter,
                "line {line} of its journal comes after the search finished"
            ),
        }
    }
}

impl Error for JournalError {}
