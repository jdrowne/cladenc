use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::{FORMAT, JournalError, Record, Settings, path, whole_lines};
use crate::genealogy::{Experiment, Genealogy, HarnessId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Started {
    Procreation {
        child: HarnessId,
        progenitor: HarnessId,
    },
    Baseline {
        harness: HarnessId,
        child: HarnessId,
        progenitor: HarnessId,
    },
    Attempt {
        harness: HarnessId,
        task: String,
    },
}

impl Display for Started {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Procreation { child, progenitor } => write!(
                formatter,
                "the procreation of harness {child} by harness {progenitor}"
            ),
            Self::Baseline {
                harness,
                progenitor,
                ..
            } => write!(
                formatter,
                "the creation of harness {harness}, the singleton control group harness, by the \
                 primogenitor from the source code of harness {progenitor}"
            ),
            Self::Attempt { harness, task } => {
                write!(formatter, "the attempt of harness {harness} on task {task}")
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "record", rename_all = "snake_case")]
enum First {
    Run { format: u32 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Seconds {
    pub(crate) attempts: f64,
    pub(crate) procreation: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct State {
    pub(crate) settings: Settings,
    pub(crate) genealogy: Genealogy,
    pub(crate) unfinished: Vec<Started>,
    pub(crate) next_action: u64,
    pub(crate) best_harness: Option<HarnessId>,
    pub(crate) seconds: Vec<Seconds>,
}

pub(crate) fn read(outputs_directory: &Path) -> Result<State, JournalError> {
    let contents = fs::read(path(outputs_directory)).map_err(JournalError::Io)?;
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
        seconds: vec![Seconds::default()],
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
            Record::ProcreationStarted { child, progenitor } => {
                if child != self.genealogy.harness_count() {
                    return Err(JournalError::WrongChild { line, child });
                }
                self.genealogy
                    .start_procreation(progenitor)
                    .map_err(refused)?;
                self.seconds.push(Seconds::default());
                self.start(Started::Procreation { child, progenitor });
            }
            Record::ProcreationFinished {
                child,
                outcome,
                seconds,
            } => {
                self.genealogy
                    .end_procreation(child, outcome)
                    .map_err(refused)?;
                self.seconds[child as usize].procreation = Some(seconds);
                self.unfinished
                    .retain(|action| !matches!(action, Started::Procreation { child: running, .. } if *running == child));
            }
            Record::BaselineStarted {
                harness,
                child,
                progenitor,
            } => {
                if harness != self.genealogy.harness_count() {
                    return Err(JournalError::WrongChild {
                        line,
                        child: harness,
                    });
                }
                self.genealogy
                    .start_baseline(Experiment { child, progenitor })
                    .map_err(refused)?;
                self.seconds.push(Seconds::default());
                self.start(Started::Baseline {
                    harness,
                    child,
                    progenitor,
                });
            }
            Record::BaselineFinished {
                harness, outcome, ..
            } => {
                self.genealogy
                    .end_baseline(harness, outcome)
                    .map_err(refused)?;
                self.unfinished
                    .retain(|action| !matches!(action, Started::Baseline { harness: running, .. } if *running == harness));
            }
            Record::AttemptStarted { harness, task } => {
                self.genealogy
                    .start_attempt(harness, &task)
                    .map_err(refused)?;
                self.start(Started::Attempt { harness, task });
            }
            Record::AttemptFinished {
                harness,
                task,
                result,
                seconds,
            } => {
                self.genealogy
                    .end_attempt(harness, &task, result)
                    .map_err(refused)?;
                self.seconds[harness as usize].attempts += seconds;
                let finished = Started::Attempt { harness, task };
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
        self.next_action = self.genealogy.actions_started();
    }
}
