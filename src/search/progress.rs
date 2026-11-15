use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::Scope;

use super::said::say;
use super::worker::{self, Ended};
use super::{Found, Said, SearchError, Shared};
use crate::genealogy::Genealogy;
use crate::job::JobError;
use crate::journal::{Journal, Record, Settings, Started};
use crate::policy::{Job, Policy};
use crate::workspace::Workspace;

pub(super) struct Progress<'a> {
    policy: Policy,
    journal: Journal,
    writer: Option<&'a mut dyn Write>,
    genealogy: Genealogy,
    again: VecDeque<Started>,
    next_job: u64,
    initial_left: u64,
    initial_ended: bool,
    running: u64,
    failure: Option<SearchError>,
    stopping: bool,
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
            genealogy: settings.genealogy(),
            again: VecDeque::new(),
            next_job: 0,
            initial_left: settings.initial_procreations,
            initial_ended: false,
            running: 0,
            failure: None,
            stopping: false,
        }
    }

    pub(super) fn say(&mut self, said: &Said<'_>) {
        say(&mut self.writer, said);
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
        unfinished: Vec<(u64, Started)>,
        next_job: u64,
    ) -> Self {
        let initial = self.initial_left;
        self.initial_left = initial.saturating_sub(next_job);
        self.initial_ended =
            self.initial_left == 0 && unfinished.iter().all(|(job, _)| *job >= initial);
        self.genealogy = genealogy;
        self.again = unfinished.into_iter().map(|(_, job)| job).collect();
        self.next_job = next_job;
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
            self.fail(SearchError::Journal(error));
        }
    }

    pub(super) fn take(&mut self) -> Option<(Started, bool)> {
        self.again
            .pop_front()
            .map(|job| (job, true))
            .or_else(|| self.next().map(|job| (self.start(job), false)))
    }

    pub(super) fn hand_over<'scope>(
        &mut self,
        job: Started,
        again: bool,
        shared: &'scope Shared<'_>,
        scope: &'scope Scope<'scope, '_>,
        sender: &Sender<Ended>,
    ) {
        let recorded = self.failure.is_none();
        let sender = sender.clone();
        let _ = recorded.then(|| {
            say(
                &mut self.writer,
                &Said::Started {
                    job: &job,
                    again,
                    evaluations: self.genealogy.started(),
                    budget: self.policy.budget,
                },
            );
            scope.spawn(move || {
                let _ = sender.send(worker::carry_out(shared, job, again));
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
            Ended::Evaluation {
                harness,
                task,
                result: Ok(outcome),
            } => {
                self.genealogy
                    .end_evaluation(harness, &task, outcome)
                    .expect("an evaluation that ended was started");
                self.say(&Said::Evaluated {
                    harness,
                    task: &task,
                    outcome,
                    workspace,
                });
                self.record(&Record::EvaluationFinished {
                    harness,
                    task,
                    outcome,
                });
            }
            Ended::Procreation {
                result: Err(JobError::Stopped),
                ..
            }
            | Ended::Evaluation {
                result: Err(JobError::Stopped),
                ..
            } => {}
            Ended::Procreation {
                child,
                parent,
                result: Err(error),
            } => self.failed(Started::Procreation { child, parent }, error),
            Ended::Evaluation {
                harness,
                task,
                result: Err(error),
            } => self.failed(Started::Evaluation { harness, task }, error),
        }
    }

    fn failed(&mut self, job: Started, error: JobError) {
        self.say(&Said::Failed {
            job: &job,
            error: &error,
        });
        self.fail(SearchError::Job { job, error });
    }

    pub(super) fn finish(
        mut self,
        workspace: &Workspace,
        stop: &AtomicBool,
    ) -> Result<Found, SearchError> {
        if self.halted(stop) {
            return Err(self.failure.unwrap_or(SearchError::Asked));
        }
        let best_harness = self.policy.best_harness(&self.genealogy);
        self.record(&Record::SearchFinished { best_harness });
        let found = Found {
            best_harness,
            directory: workspace.harness(best_harness),
        };
        let _ = self.failure.is_none().then(|| {
            say(
                &mut self.writer,
                &Said::Finished {
                    evaluations: self.genealogy.started(),
                    harnesses: self.genealogy.harness_count(),
                    best_harness,
                    directory: &found.directory,
                },
            );
        });
        self.failure.map_or(Ok(found), Err)
    }

    fn next(&mut self) -> Option<Job> {
        if self.initial_left > 0 {
            self.initial_left -= 1;
            return Some(Job::Procreation { parent: 0 });
        }
        self.initial_ended = self.initial_ended || self.running == 0;
        self.initial_ended
            .then(|| self.policy.next_job(&self.genealogy, self.next_job))
            .flatten()
    }

    fn start(&mut self, job: Job) -> Started {
        let started = match job {
            Job::Procreation { parent } => {
                let child = self
                    .genealogy
                    .start_procreation(parent)
                    .expect("the parent of a procreation is a created harness");
                self.record(&Record::ProcreationStarted { child, parent });
                Started::Procreation { child, parent }
            }
            Job::Evaluation { harness, task } => {
                self.genealogy
                    .start_evaluation(harness, &task)
                    .expect("an evaluated harness is created and has not been given the task");
                self.record(&Record::EvaluationStarted {
                    harness,
                    task: task.clone(),
                });
                Started::Evaluation { harness, task }
            }
        };
        self.next_job += 1;
        started
    }

    fn fail(&mut self, error: SearchError) {
        self.failure = self.failure.take().or(Some(error));
    }
}
