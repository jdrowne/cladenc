use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::Scope;

use super::action::{Ended, carry_out};
use super::said::say;
use super::{DirectedEvolutionError, Found, Said, Shared};
use crate::artificial_selection::{Action, ArtificialSelection};
use crate::containers::ContainerError;
use crate::genealogy::Genealogy;
use crate::journal::{Journal, Record, Settings, Started};
use crate::workspace::Workspace;

pub(super) struct Progress<'a> {
    artificial_selection: ArtificialSelection,
    journal: Journal,
    writer: &'a mut dyn Write,
    genealogy: Genealogy,
    again: VecDeque<Started>,
    next_action: u64,
    running: u64,
    failure: Option<DirectedEvolutionError>,
    stopping: bool,
}

impl<'a> Progress<'a> {
    pub(super) fn new(settings: &Settings, journal: Journal, writer: &'a mut dyn Write) -> Self {
        Self {
            artificial_selection: settings.artificial_selection(),
            journal,
            writer,
            genealogy: settings.genealogy(),
            again: VecDeque::new(),
            next_action: 0,
            running: 0,
            failure: None,
            stopping: false,
        }
    }

    pub(super) fn say(&mut self, said: &Said<'_>) {
        say(self.writer, said);
    }

    pub(super) fn stopping(&mut self) {
        if !self.stopping {
            self.stopping = true;
            self.say(&Said::Stopping {
                asked: self.failure.is_none(),
                running: self.running,
            });
        }
    }

    pub(super) fn resumed(
        mut self,
        genealogy: Genealogy,
        unfinished: Vec<Started>,
        next_action: u64,
    ) -> Self {
        self.genealogy = genealogy;
        self.again = unfinished.into();
        self.next_action = next_action;
        self
    }

    pub(super) fn running(&self) -> u64 {
        self.running
    }

    pub(super) fn halted(&self, stop: &AtomicBool) -> bool {
        self.failure.is_some() || stop.load(Ordering::SeqCst)
    }

    pub(super) fn awaits_build(&self, stop: &AtomicBool) -> bool {
        self.genealogy.awaits_build() && !self.halted(stop)
    }

    pub(super) fn built(&mut self) {
        self.genealogy
            .build_progenitor()
            .expect("a progenitor that was built was waiting to be");
        self.record(&Record::ProgenitorBuilt);
        self.say(&Said::Built);
    }

    pub(super) fn record(&mut self, record: &Record) {
        if let Err(error) = self.journal.append(record) {
            self.fail(DirectedEvolutionError::Journal(error));
        }
    }

    pub(super) fn take(&mut self) -> Option<(Started, bool)> {
        self.again
            .pop_front()
            .map(|action| (action, true))
            .or_else(|| {
                self.artificial_selection
                    .next_action(&self.genealogy, self.next_action)
                    .map(|action| (self.start(action), false))
            })
    }

    pub(super) fn hand_over<'scope>(
        &mut self,
        action: Started,
        again: bool,
        shared: &'scope Shared<'_>,
        scope: &'scope Scope<'scope, '_>,
        sender: &Sender<Ended>,
    ) {
        let recorded = self.failure.is_none();
        let sender = sender.clone();
        let _ = recorded.then(|| {
            say(
                self.writer,
                &Said::Started {
                    action: &action,
                    again,
                    assessments: self.genealogy.started(),
                    budget: self.artificial_selection.budget,
                },
            );
            scope.spawn(move || {
                let _ = sender.send(carry_out(shared, action, again));
            })
        });
        self.running += u64::from(recorded);
    }

    pub(super) fn end(&mut self, ended: Ended, workspace: &Workspace) {
        self.running -= 1;
        match ended {
            Ended::Procreation {
                child,
                parent,
                result: Ok(outcome),
            } => {
                self.genealogy
                    .end_procreation(child, outcome)
                    .expect("a procreation that ended was started");
                self.record(&Record::ProcreationFinished { child, outcome });
                self.say(&Said::Procreated {
                    child,
                    parent,
                    outcome,
                    workspace,
                });
            }
            Ended::Assessment {
                harness,
                task,
                result: Ok(score),
            } => {
                self.genealogy
                    .end_assessment(harness, &task, score)
                    .expect("an assessment that ended was started");
                self.say(&Said::Scored {
                    harness,
                    task: &task,
                    score,
                    workspace,
                });
                self.record(&Record::AssessmentFinished {
                    harness,
                    task,
                    score,
                });
            }
            Ended::Procreation {
                result: Err(ContainerError::Stopped),
                ..
            }
            | Ended::Assessment {
                result: Err(ContainerError::Stopped),
                ..
            } => {}
            Ended::Procreation {
                child,
                parent,
                result: Err(error),
            } => self.failed(Started::Procreation { child, parent }, error),
            Ended::Assessment {
                harness,
                task,
                result: Err(error),
            } => self.failed(Started::Assessment { harness, task }, error),
        }
    }

    fn failed(&mut self, action: Started, error: ContainerError) {
        self.say(&Said::Failed {
            action: &action,
            error: &error,
        });
        self.fail(DirectedEvolutionError::Action { action, error });
    }

    pub(super) fn finish(
        mut self,
        workspace: &Workspace,
        stop: &AtomicBool,
    ) -> Result<Found, DirectedEvolutionError> {
        if self.halted(stop) {
            return Err(self.failure.unwrap_or(DirectedEvolutionError::Asked));
        }
        let best_harness = self.artificial_selection.best_harness(&self.genealogy);
        self.record(&Record::DirectedEvolutionFinished { best_harness });
        let found = Found {
            best_harness,
            directory: workspace.harness(best_harness),
        };
        let _ = self.failure.is_none().then(|| {
            say(
                self.writer,
                &Said::Finished {
                    assessments: self.genealogy.started(),
                    harnesses: self.genealogy.harness_count(),
                    best_harness,
                    directory: &found.directory,
                },
            );
        });
        self.failure.map_or(Ok(found), Err)
    }

    fn start(&mut self, action: Action) -> Started {
        let started = match action {
            Action::Procreation { parent } => {
                let child = self
                    .genealogy
                    .start_procreation(parent)
                    .expect("the parent of a procreation is a created harness");
                self.record(&Record::ProcreationStarted { child, parent });
                Started::Procreation { child, parent }
            }
            Action::Assessment { harness, task } => {
                self.genealogy
                    .start_assessment(harness, &task)
                    .expect("an assessed harness is created and has not been given the task");
                self.record(&Record::AssessmentStarted {
                    harness,
                    task: task.clone(),
                });
                Started::Assessment { harness, task }
            }
        };
        self.next_action += 1;
        started
    }

    fn fail(&mut self, error: DirectedEvolutionError) {
        self.failure = self.failure.take().or(Some(error));
    }
}
