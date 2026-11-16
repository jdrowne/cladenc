use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::{FORMAT, JournalError, Record, Settings, path, whole_lines};
use crate::genealogy::{Genealogy, HarnessId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Started {
    Procreation { child: HarnessId, parent: HarnessId },
    Assessment { harness: HarnessId, task: String },
}

impl Display for Started {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Procreation { child, parent } => write!(
                formatter,
                "the procreation of harness {child} by harness {parent}"
            ),
            Self::Assessment { harness, task } => write!(
                formatter,
                "the assessment of harness {harness} on task {task}"
            ),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "record", rename_all = "snake_case")]
enum First {
    Run { format: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub settings: Settings,
    pub genealogy: Genealogy,
    pub unfinished: Vec<Started>,
    pub next_action: u64,
    pub best_harness: Option<HarnessId>,
}

pub fn read(run_directory: &Path) -> Result<State, JournalError> {
    let contents = fs::read(path(run_directory)).map_err(JournalError::Io)?;
    let lines = || whole_lines(&contents).split_inclusive(|byte| *byte == b'\n');
    if let Some(First::Run { format }) = lines()
        .next()
        .and_then(|first| serde_json::from_slice(first).ok())
        && format != FORMAT
    {
        return Err(JournalError::UnknownFormat { format });
    }
    let mut records = lines().zip(1..).map(|(text, line)| {
        serde_json::from_slice::<Record>(text)
            .map(|record| (line, record))
            .map_err(|_| JournalError::NotARecord { line })
    });
    let settings = match records.next() {
        Some(Ok((_, Record::Run { settings, .. }))) => *settings,
        Some(Err(error)) => return Err(error),
        _ => return Err(JournalError::FirstRecordNotRun),
    };
    let mut state = State {
        genealogy: settings.genealogy(),
        settings,
        unfinished: Vec::new(),
        next_action: 0,
        best_harness: None,
    };
    for record in records {
        let (line, record) = record?;
        if state.best_harness.is_some() {
            return Err(JournalError::AfterDirectedEvolutionFinished { line });
        }
        state.apply(line, record)?;
    }
    Ok(state)
}

impl State {
    fn apply(&mut self, line: usize, record: Record) -> Result<(), JournalError> {
        let refused = |error| JournalError::Refused { line, error };
        match record {
            Record::Run { .. } => return Err(JournalError::RunRecordRepeated { line }),
            Record::PrimogenitorBuilt => self.genealogy.build_primogenitor().map_err(refused)?,
            Record::ProcreationStarted { child, parent } => {
                if child != self.genealogy.harness_count() {
                    return Err(JournalError::WrongChild { line, child });
                }
                self.genealogy.start_procreation(parent).map_err(refused)?;
                self.start(Started::Procreation { child, parent });
            }
            Record::ProcreationFinished { child, outcome } => {
                self.genealogy
                    .end_procreation(child, outcome)
                    .map_err(refused)?;
                self.unfinished
                    .retain(|action| !matches!(action, Started::Procreation { child: running, .. } if *running == child));
            }
            Record::AssessmentStarted { harness, task } => {
                self.genealogy
                    .start_assessment(harness, &task)
                    .map_err(refused)?;
                self.start(Started::Assessment { harness, task });
            }
            Record::AssessmentFinished {
                harness,
                task,
                score,
            } => {
                self.genealogy
                    .end_assessment(harness, &task, score)
                    .map_err(refused)?;
                let finished = Started::Assessment { harness, task };
                self.unfinished.retain(|action| *action != finished);
            }
            Record::DirectedEvolutionFinished { best_harness } => {
                self.best_harness = Some(best_harness)
            }
        }
        Ok(())
    }

    fn start(&mut self, action: Started) {
        self.unfinished.push(action);
        self.next_action += 1;
    }
}
