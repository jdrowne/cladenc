mod copy;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::genealogy::{EvaluationOutcome, HarnessId, ProcreationOutcome};

pub use copy::{copy_tree, holds_only_harness_files};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    run_directory: PathBuf,
}

impl Workspace {
    pub fn at(run_directory: &Path) -> Self {
        Self {
            run_directory: run_directory.to_owned(),
        }
    }

    pub fn create(run_directory: &Path, progenitor: &Path) -> io::Result<Self> {
        let workspace = Self::at(run_directory);
        fs::create_dir(run_directory)
            .and_then(|()| fs::create_dir(run_directory.join("harnesses")))
            .and_then(|()| fs::create_dir(run_directory.join("procreations")))
            .and_then(|()| fs::create_dir(run_directory.join("evaluations")))
            .and_then(|()| fs::create_dir(run_directory.join("running")))
            .and_then(|()| fs::canonicalize(progenitor))
            .and_then(|resolved| copy_tree(&resolved, &workspace.harness(0)))
            .and_then(|()| fs::create_dir(workspace.evaluations(0)))
            .map(|()| workspace)
    }

    pub fn harness(&self, id: HarnessId) -> PathBuf {
        self.run_directory.join("harnesses").join(id.to_string())
    }

    pub fn procreation(&self, child: HarnessId) -> PathBuf {
        self.run_directory
            .join("procreations")
            .join(child.to_string())
    }

    pub fn evaluations(&self, harness: HarnessId) -> PathBuf {
        self.run_directory
            .join("evaluations")
            .join(harness.to_string())
    }

    pub fn evaluation(&self, harness: HarnessId, task: &str) -> PathBuf {
        self.evaluations(harness).join(task)
    }

    pub fn running_procreation(&self, child: HarnessId) -> PathBuf {
        self.run_directory
            .join("running")
            .join(format!("procreation-{child}"))
    }

    pub fn running_evaluation(&self, harness: HarnessId, task: &str) -> PathBuf {
        self.run_directory
            .join("running")
            .join(format!("evaluation-{harness}-{task}"))
    }

    pub fn start_procreation(&self, child: HarnessId, parent: HarnessId) -> io::Result<()> {
        let job = self.running_procreation(child);
        let harness = self.harness(parent);
        fs::create_dir(&job)
            .and_then(|()| fs::create_dir(job.join("output")))
            .and_then(|()| copy_tree(&harness, &job.join("parent")))
            .and_then(|()| copy_tree(&harness, &job.join("child")))
    }

    pub fn end_procreation(&self, child: HarnessId, outcome: ProcreationOutcome) -> io::Result<()> {
        let job = self.running_procreation(child);
        let made = job.join("child");
        let settled = match outcome {
            ProcreationOutcome::Created => fs::rename(made, self.harness(child))
                .and_then(|()| fs::create_dir(self.evaluations(child))),
            ProcreationOutcome::Failed | ProcreationOutcome::TimedOut => fs::remove_dir_all(made),
        };
        settled
            .and_then(|()| fs::remove_dir_all(job.join("parent")))
            .and_then(|()| fs::rename(&job, self.procreation(child)))
    }

    pub fn start_evaluation(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        let job = self.running_evaluation(harness, task);
        fs::create_dir(&job)
            .and_then(|()| fs::create_dir(job.join("output")))
            .and_then(|()| copy_tree(&self.harness(harness), &job.join("harness")))
    }

    pub fn end_evaluation(
        &self,
        harness: HarnessId,
        task: &str,
        outcome: EvaluationOutcome,
    ) -> io::Result<()> {
        let job = self.running_evaluation(harness, task);
        let word = match outcome {
            EvaluationOutcome::Success => "success\n",
            EvaluationOutcome::Failure => "failure\n",
            EvaluationOutcome::TimedOut => "timed_out\n",
        };
        fs::remove_dir_all(job.join("harness"))
            .and_then(|()| fs::write(job.join("outcome"), word))
            .and_then(|()| fs::rename(&job, self.evaluation(harness, task)))
    }

    pub fn discard_procreation(&self, child: HarnessId) -> io::Result<()> {
        [
            self.running_procreation(child),
            self.harness(child),
            self.procreation(child),
            self.evaluations(child),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(left))
    }

    pub fn discard_evaluation(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        [
            self.running_evaluation(harness, task),
            self.evaluation(harness, task),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(left))
    }
}

fn remove_if_there(directory: &Path) -> io::Result<()> {
    if fs::symlink_metadata(directory).is_ok() {
        fs::remove_dir_all(directory)
    } else {
        Ok(())
    }
}
