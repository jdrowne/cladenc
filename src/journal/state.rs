use std::fs;
use std::path::Path;

use super::{FORMAT, JournalError, Record, Settings, path, whole_lines};
use crate::genealogy::{Genealogy, HarnessId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Started {
    Expansion { child: HarnessId, parent: HarnessId },
    Evaluation { harness: HarnessId, task: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub settings: Settings,
    pub genealogy: Genealogy,
    pub unfinished: Vec<(u64, Started)>,
    pub next_job: u64,
    pub best_harness: Option<HarnessId>,
}

pub fn read(run_directory: &Path) -> Result<State, JournalError> {
    let contents = fs::read(path(run_directory)).map_err(JournalError::Io)?;
    let mut records = whole_lines(&contents)
        .split_inclusive(|byte| *byte == b'\n')
        .zip(1..)
        .map(|(text, line)| {
            serde_json::from_slice::<Record>(text)
                .map(|record| (line, record))
                .map_err(|_| JournalError::NotARecord { line })
        });
    let settings = match records.next() {
        Some(Ok((_, Record::Run { format, settings }))) if format == FORMAT => settings,
        Some(Ok((_, Record::Run { format, .. }))) => {
            return Err(JournalError::UnknownFormat { format });
        }
        Some(Err(error)) => return Err(error),
        _ => return Err(JournalError::FirstRecordNotRun),
    };
    let mut state = State {
        settings,
        genealogy: Genealogy::new(),
        unfinished: Vec::new(),
        next_job: 0,
        best_harness: None,
    };
    for record in records {
        let (line, record) = record?;
        if state.best_harness.is_some() {
            return Err(JournalError::AfterSearchFinished { line });
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
            Record::ExpansionStarted { child, parent } => {
                if child != self.genealogy.harness_count() {
                    return Err(JournalError::WrongChild { line, child });
                }
                self.genealogy.start_expansion(parent).map_err(refused)?;
                self.start(Started::Expansion { child, parent });
            }
            Record::ExpansionFinished { child, outcome } => {
                self.genealogy
                    .end_expansion(child, outcome)
                    .map_err(refused)?;
                self.unfinished
                    .retain(|(_, job)| !matches!(job, Started::Expansion { child: running, .. } if *running == child));
            }
            Record::EvaluationStarted { harness, task } => {
                self.genealogy
                    .start_evaluation(harness, &task)
                    .map_err(refused)?;
                self.start(Started::Evaluation { harness, task });
            }
            Record::EvaluationFinished {
                harness,
                task,
                outcome,
            } => {
                self.genealogy
                    .end_evaluation(harness, &task, outcome)
                    .map_err(refused)?;
                let finished = Started::Evaluation { harness, task };
                self.unfinished.retain(|(_, job)| *job != finished);
            }
            Record::SearchFinished { best_harness } => self.best_harness = Some(best_harness),
        }
        Ok(())
    }

    fn start(&mut self, job: Started) {
        self.unfinished.push((self.next_job, job));
        self.next_job += 1;
    }
}
