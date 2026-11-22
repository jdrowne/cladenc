use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Scope;

use super::action::{Ended, carry_out};
use super::said::say;
use super::{DirectedEvolutionError, Found, Said, Shared};
use crate::artificial_selection::{Action, ArtificialSelection};
use crate::containers::{Attempted, ContainerError, Creation};
use crate::genealogy::{Experiment, Genealogy};
use crate::journal::{Journal, Record, Settings, Started};
use crate::outputs_directory::OutputsDirectory;

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
            .build_primogenitor()
            .expect("a primogenitor that was built was waiting to be");
        self.record(&Record::PrimogenitorBuilt);
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
                self.genealogy
                    .experiment_awaiting_its_baseline()
                    .map(|experiment| (self.start_baseline(experiment), false))
            })
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
    ) {
        let recorded = self.failure.is_none();
        let _ = recorded.then(|| {
            say(
                self.writer,
                &Said::Started {
                    action: &action,
                    again,
                    procreations: self.genealogy.procreations_started(),
                    max_procreations: self.artificial_selection.max_procreations,
                },
            );
            scope.spawn(move || {
                let _ = shared.sender.send(carry_out(shared, action, again));
            })
        });
        self.running += u64::from(recorded);
    }

    pub(super) fn end(&mut self, ended: Ended, outputs_directory: &OutputsDirectory) {
        self.running -= 1;
        match ended {
            Ended::Procreated {
                child,
                progenitor,
                creation: Creation { outcome, seconds },
            } => {
                self.genealogy
                    .end_procreation(child, outcome)
                    .expect("a procreation that ended was started");
                self.record(&Record::ProcreationFinished {
                    child,
                    outcome,
                    seconds,
                });
                self.say(&Said::Procreated {
                    child,
                    progenitor,
                    outcome,
                    outputs_directory,
                });
            }
            Ended::Established {
                harness,
                experiment,
                creation: Creation { outcome, seconds },
            } => {
                self.genealogy
                    .end_baseline(harness, outcome)
                    .expect("a baseline that ended was started");
                self.record(&Record::BaselineFinished {
                    harness,
                    outcome,
                    seconds,
                });
                self.say(&Said::Established {
                    harness,
                    experiment,
                    outcome,
                    outputs_directory,
                });
            }
            Ended::Attempted {
                harness,
                task,
                attempted:
                    Attempted {
                        result,
                        seconds,
                        ran_out_of_time,
                    },
            } => {
                self.genealogy
                    .end_attempt(harness, &task, result)
                    .expect("an attempt that ended was started");
                self.say(&Said::Attempted {
                    harness,
                    task: &task,
                    result,
                    ran_out_of_time,
                    outputs_directory,
                });
                self.record(&Record::AttemptFinished {
                    harness,
                    task,
                    result,
                    seconds,
                });
            }
            Ended::Stopped => {}
            Ended::Failed { action, error } => self.failed(action, error),
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
        outputs_directory: &OutputsDirectory,
        stop: &AtomicBool,
    ) -> Result<Found, DirectedEvolutionError> {
        if self.halted(stop) {
            return Err(self.failure.unwrap_or(DirectedEvolutionError::Asked));
        }
        let best_harness = self.artificial_selection.best_harness(&self.genealogy);
        self.record(&Record::DirectedEvolutionFinished { best_harness });
        let found = Found {
            best_harness,
            directory: outputs_directory.harness(best_harness),
        };
        let _ = self.failure.is_none().then(|| {
            say(
                self.writer,
                &Said::Finished {
                    attempts: self.genealogy.started(),
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
            Action::Procreation { progenitor } => {
                let child = self
                    .genealogy
                    .start_procreation(progenitor)
                    .expect("the progenitor of a procreation is a created harness");
                self.record(&Record::ProcreationStarted { child, progenitor });
                Started::Procreation { child, progenitor }
            }
            Action::Attempt { harness, task } => {
                self.genealogy.start_attempt(harness, &task).expect(
                    "a harness that attempts a task is created and has not been given the task",
                );
                self.record(&Record::AttemptStarted {
                    harness,
                    task: task.clone(),
                });
                Started::Attempt { harness, task }
            }
        };
        self.next_action = self.genealogy.actions_started();
        started
    }

    fn start_baseline(&mut self, experiment: Experiment) -> Started {
        let Experiment { child, progenitor } = experiment;
        let harness = self
            .genealogy
            .start_baseline(experiment)
            .expect("the experiment of a baseline awaits it");
        self.record(&Record::BaselineStarted {
            harness,
            child,
            progenitor,
        });
        Started::Baseline {
            harness,
            child,
            progenitor,
        }
    }

    fn fail(&mut self, error: DirectedEvolutionError) {
        self.failure = self.failure.take().or(Some(error));
    }
}
