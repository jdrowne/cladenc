use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::Scope;

use super::{Found, SearchError, Shared};
use crate::genealogy::{EvaluationOutcome, ExpansionOutcome, Genealogy, HarnessId};
use crate::job::{self, JobError};
use crate::journal::{Journal, Record, Settings};
use crate::policy::{Job, Policy};
use crate::workspace::Workspace;

pub(super) enum Ended {
    Expansion {
        child: HarnessId,
        result: Result<ExpansionOutcome, JobError>,
    },
    Evaluation {
        harness: HarnessId,
        task: String,
        result: Result<EvaluationOutcome, JobError>,
    },
}

pub(super) struct Progress<'a> {
    policy: Policy,
    journal: Journal,
    writer: Option<&'a mut dyn Write>,
    genealogy: Genealogy,
    next_job: u64,
    initial_left: u64,
    initial_ended: bool,
    running: u64,
    failure: Option<SearchError>,
}

impl<'a> Progress<'a> {
    pub(super) fn new(
        settings: &Settings,
        journal: Journal,
        writer: Option<&'a mut dyn Write>,
    ) -> Self {
        Self {
            policy: settings.policy(),
            journal,
            writer,
            genealogy: Genealogy::new(),
            next_job: 0,
            initial_left: settings.initial_expansions,
            initial_ended: false,
            running: 0,
            failure: None,
        }
    }

    pub(super) fn running(&self) -> u64 {
        self.running
    }

    pub(super) fn halted(&self, stop: &AtomicBool) -> bool {
        self.failure.is_some() || stop.load(Ordering::SeqCst)
    }

    pub(super) fn record(&mut self, record: &Record) {
        let written = record
            .line()
            .and_then(|line| self.journal.append(record).map(|()| line));
        match written {
            Ok(line) => {
                if let Some(writer) = self.writer.as_mut() {
                    let _ = writeln!(writer, "{line}");
                }
            }
            Err(error) => self.fail(SearchError::Journal(error)),
        }
    }

    pub(super) fn next(&mut self) -> Option<Job> {
        if self.initial_left > 0 {
            self.initial_left -= 1;
            return Some(Job::Expansion { parent: 0 });
        }
        self.initial_ended = self.initial_ended || self.running == 0;
        self.initial_ended
            .then(|| self.policy.next_job(&self.genealogy, self.next_job))
            .flatten()
    }

    pub(super) fn start<'scope>(
        &mut self,
        job: Job,
        shared: &'scope Shared<'_>,
        scope: &'scope Scope<'scope, '_>,
        sender: &Sender<Ended>,
    ) {
        let sender = sender.clone();
        let settings = shared.settings;
        match job {
            Job::Expansion { parent } => {
                let child = self
                    .genealogy
                    .start_expansion(parent)
                    .expect("the parent of an expansion is a created harness");
                self.record(&Record::ExpansionStarted { child, parent });
                self.hand_over(scope, move || {
                    let result = job::run_expansion(
                        &shared.workspace,
                        &settings.expander,
                        child,
                        parent,
                        settings.expansion_timeout,
                        &shared.halt,
                    );
                    let _ = sender.send(Ended::Expansion { child, result });
                });
            }
            Job::Evaluation { harness, task } => {
                self.genealogy
                    .start_evaluation(harness, &task)
                    .expect("an evaluated harness is created and has not been given the task");
                self.record(&Record::EvaluationStarted {
                    harness,
                    task: task.clone(),
                });
                self.hand_over(scope, move || {
                    let result = job::run_evaluation(
                        &shared.workspace,
                        &settings.evaluator,
                        harness,
                        &task,
                        settings.evaluation_timeout,
                        &shared.halt,
                    );
                    let _ = sender.send(Ended::Evaluation {
                        harness,
                        task,
                        result,
                    });
                });
            }
        }
        self.next_job += 1;
    }

    pub(super) fn end(&mut self, ended: Ended) {
        self.running -= 1;
        match ended {
            Ended::Expansion {
                child,
                result: Ok(outcome),
            } => {
                self.genealogy
                    .end_expansion(child, outcome)
                    .expect("an expansion that ended was started");
                self.record(&Record::ExpansionFinished { child, outcome });
            }
            Ended::Evaluation {
                harness,
                task,
                result: Ok(outcome),
            } => {
                self.genealogy
                    .end_evaluation(harness, &task, outcome)
                    .expect("an evaluation that ended was started");
                self.record(&Record::EvaluationFinished {
                    harness,
                    task,
                    outcome,
                });
            }
            Ended::Expansion {
                result: Err(JobError::Stopped),
                ..
            }
            | Ended::Evaluation {
                result: Err(JobError::Stopped),
                ..
            } => {}
            Ended::Expansion {
                result: Err(error), ..
            }
            | Ended::Evaluation {
                result: Err(error), ..
            } => self.fail(SearchError::Stopped(Some(error))),
        }
    }

    pub(super) fn finish(
        mut self,
        workspace: &Workspace,
        stop: &AtomicBool,
    ) -> Result<Found, SearchError> {
        if self.halted(stop) {
            return Err(self.failure.unwrap_or(SearchError::Stopped(None)));
        }
        let best_harness = self.policy.best_harness(&self.genealogy);
        self.record(&Record::SearchFinished { best_harness });
        let found = Found {
            best_harness,
            directory: workspace.harness(best_harness),
        };
        self.failure.map_or(Ok(found), Err)
    }

    fn hand_over<'scope>(
        &mut self,
        scope: &'scope Scope<'scope, '_>,
        work: impl FnOnce() + Send + 'scope,
    ) {
        let recorded = self.failure.is_none();
        let _ = recorded.then(|| scope.spawn(work));
        self.running += u64::from(recorded);
    }

    fn fail(&mut self, error: SearchError) {
        self.failure = self.failure.take().or(Some(error));
    }
}
