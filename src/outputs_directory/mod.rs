mod copy;

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, HarnessId};

pub use copy::{copy_tree, holds_only_harness_files};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputsDirectory {
    path: PathBuf,
    holdout_tasks: BTreeSet<String>,
}

impl OutputsDirectory {
    pub fn at(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            holdout_tasks: BTreeSet::new(),
        }
    }

    pub fn holding_out(self, tasks: &[String]) -> Self {
        Self {
            holdout_tasks: tasks.iter().cloned().collect(),
            ..self
        }
    }

    pub fn create(path: &Path, primogenitor: &Path) -> io::Result<Self> {
        let outputs_directory = Self::at(path);
        fs::create_dir(path)
            .and_then(|()| fs::create_dir(path.join("harnesses")))
            .and_then(|()| fs::create_dir(path.join("procreations")))
            .and_then(|()| fs::create_dir(path.join("baselines")))
            .and_then(|()| fs::create_dir(path.join("attempts")))
            .and_then(|()| fs::create_dir(path.join("holdout")))
            .and_then(|()| fs::create_dir(path.join("running")))
            .and_then(|()| fs::create_dir(path.join("benchmarks")))
            .and_then(|()| fs::canonicalize(primogenitor))
            .and_then(|resolved| copy_tree(&resolved, &outputs_directory.harness(0)))
            .and_then(|()| fs::create_dir(outputs_directory.attempts(0)))
            .map(|()| outputs_directory)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn copy_benchmark(&self, benchmark: &Benchmark) -> io::Result<()> {
        let kept = self.path.join("benchmarks").join(benchmark.name());
        fs::create_dir(&kept).and_then(|()| {
            benchmark.tasks.iter().try_for_each(|id| {
                fs::canonicalize(benchmark.directory.join(id))
                    .and_then(|resolved| copy_tree(&resolved, &kept.join(id)))
            })
        })
    }

    pub fn task(&self, task: &str) -> PathBuf {
        self.path.join("benchmarks").join(task)
    }

    pub fn image(&self) -> PathBuf {
        self.path.join("image")
    }

    pub fn start_image(&self, dockerfile: &Path) -> io::Result<PathBuf> {
        let image = self.image();
        fs::create_dir(&image)
            .and_then(|()| fs::create_dir(image.join("context")))
            .and_then(|()| fs::copy(dockerfile, image.join("Dockerfile")))
            .map(|_| image)
    }

    pub fn harness(&self, id: HarnessId) -> PathBuf {
        self.path.join("harnesses").join(id.to_string())
    }

    pub fn procreation(&self, child: HarnessId) -> PathBuf {
        self.path.join("procreations").join(child.to_string())
    }

    pub fn baseline(&self, harness: HarnessId) -> PathBuf {
        self.path.join("baselines").join(harness.to_string())
    }

    pub fn attempts(&self, harness: HarnessId) -> PathBuf {
        self.path.join("attempts").join(harness.to_string())
    }

    pub fn holdout(&self, harness: HarnessId) -> PathBuf {
        self.path.join("holdout").join(harness.to_string())
    }

    pub fn attempt(&self, harness: HarnessId, task: &str) -> PathBuf {
        if self.holdout_tasks.contains(task) {
            self.holdout(harness).join(task)
        } else {
            self.attempts(harness).join(task)
        }
    }

    pub fn build(&self, harness: HarnessId) -> PathBuf {
        self.path.join("builds").join(harness.to_string())
    }

    pub fn running_procreation(&self, child: HarnessId) -> PathBuf {
        self.path
            .join("running")
            .join(format!("procreation-{child}"))
    }

    pub fn running_baseline(&self, harness: HarnessId) -> PathBuf {
        self.path
            .join("running")
            .join(format!("baseline-{harness}"))
    }

    pub fn running_attempt(&self, harness: HarnessId, task: &str) -> PathBuf {
        self.path
            .join("running")
            .join(format!("attempt-{harness}-{}", task.replace('/', "+")))
    }

    pub fn start_procreation(&self, child: HarnessId, progenitor: HarnessId) -> io::Result<()> {
        let running = self.running_procreation(child);
        let harness = self.harness(progenitor);
        fs::create_dir(&running)
            .and_then(|()| copy_tree(&harness, &running.join("progenitor")))
            .and_then(|()| copy_tree(&harness, &running.join("child")))
    }

    pub fn end_procreation(
        &self,
        child: HarnessId,
        progenitor: HarnessId,
        outcome: CreationOutcome,
    ) -> io::Result<()> {
        let running = self.running_procreation(child);
        self.keep_made(&running.join("child"), child, outcome)
            .and_then(|()| fs::remove_dir_all(running.join("progenitor")))
            .and_then(|()| fs::write(running.join("progenitor"), format!("{progenitor}\n")))
            .and_then(|()| fs::write(running.join("outcome"), word_of(outcome)))
            .and_then(|()| fs::rename(&running, self.procreation(child)))
    }

    pub fn start_baseline(&self, harness: HarnessId, progenitor: HarnessId) -> io::Result<()> {
        let running = self.running_baseline(harness);
        fs::create_dir(&running)
            .and_then(|()| copy_tree(&self.harness(0), &running.join("primogenitor")))
            .and_then(|()| {
                copy_tree(
                    &self.harness(progenitor),
                    &running.join("singleton-control-group-harness"),
                )
            })
    }

    pub fn end_baseline(
        &self,
        harness: HarnessId,
        experiment: Experiment,
        outcome: CreationOutcome,
    ) -> io::Result<()> {
        let running = self.running_baseline(harness);
        let Experiment { child, progenitor } = experiment;
        self.keep_made(
            &running.join("singleton-control-group-harness"),
            harness,
            outcome,
        )
        .and_then(|()| fs::remove_dir_all(running.join("primogenitor")))
        .and_then(|()| fs::write(running.join("progenitor"), format!("{progenitor}\n")))
        .and_then(|()| fs::write(running.join("child"), format!("{child}\n")))
        .and_then(|()| fs::write(running.join("outcome"), word_of(outcome)))
        .and_then(|()| fs::rename(&running, self.baseline(harness)))
    }

    fn keep_made(
        &self,
        made: &Path,
        harness: HarnessId,
        outcome: CreationOutcome,
    ) -> io::Result<()> {
        fs::rename(made, self.harness(harness)).and_then(|()| match outcome {
            CreationOutcome::Created => fs::create_dir(self.attempts(harness)),
            CreationOutcome::Failed | CreationOutcome::BuildFailed | CreationOutcome::TimedOut => {
                Ok(())
            }
        })
    }

    pub fn progenitor_of(&self, harness: HarnessId) -> Option<HarnessId> {
        fs::read_to_string(self.procreation(harness).join("progenitor"))
            .ok()
            .and_then(|id| id.trim().parse().ok())
    }

    pub fn line_of(&self, harness: HarnessId) -> Vec<HarnessId> {
        iter::successors(self.progenitor_of(harness), |ancestor| {
            self.progenitor_of(*ancestor)
                .filter(|earlier| earlier < ancestor)
        })
        .collect()
    }

    pub fn children_of(&self, progenitor: HarnessId) -> Vec<HarnessId> {
        let mut children: Vec<HarnessId> = fs::read_dir(self.path.join("procreations"))
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|made| made.file_name().to_string_lossy().parse().ok())
            .filter(|child| self.progenitor_of(*child) == Some(progenitor))
            .collect();
        children.sort_unstable();
        children
    }

    pub fn start_build(&self, harness: HarnessId) -> io::Result<PathBuf> {
        let build = self.build(harness);
        remove_if_there(&build)
            .and_then(|()| fs::create_dir_all(&build))
            .map(|()| build)
    }

    pub fn start_attempt(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        let running = self.running_attempt(harness, task);
        let given = self.task(task);
        let kept = self.attempt(harness, task);
        fs::create_dir(&running)
            .and_then(|()| fs::create_dir(running.join("grader")))
            .and_then(|()| copy_tree(&self.harness(harness), &running.join("harness")))
            .and_then(|()| copy_tree(&given.join("workspace"), &running.join("work")))
            .and_then(|()| fs::copy(given.join("specification"), running.join("specification")))
            .and_then(|_| kept.parent().map_or(Ok(()), fs::create_dir_all))
    }

    pub fn end_attempt(
        &self,
        harness: HarnessId,
        task: &str,
        result: AttemptResult,
    ) -> io::Result<()> {
        let running = self.running_attempt(harness, task);
        let word = match result {
            AttemptResult::Completed => "completed\n",
            AttemptResult::NotCompleted => "not_completed\n",
            AttemptResult::TimedOut => "timed_out\n",
        };
        fs::remove_dir_all(running.join("harness"))
            .and_then(|()| fs::write(running.join("result"), word))
            .and_then(|()| fs::rename(&running, self.attempt(harness, task)))
    }

    pub fn discard_procreation(&self, child: HarnessId) -> io::Result<()> {
        [
            self.running_procreation(child),
            self.harness(child),
            self.procreation(child),
            self.attempts(child),
            self.build(child),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(left))
    }

    pub fn discard_baseline(&self, harness: HarnessId) -> io::Result<()> {
        [
            self.running_baseline(harness),
            self.harness(harness),
            self.baseline(harness),
            self.attempts(harness),
            self.build(harness),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(left))
    }

    pub fn discard_attempt(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        [
            self.running_attempt(harness, task),
            self.attempt(harness, task),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(left))
    }
}

fn word_of(outcome: CreationOutcome) -> &'static str {
    match outcome {
        CreationOutcome::Created => "created\n",
        CreationOutcome::Failed => "failed\n",
        CreationOutcome::BuildFailed => "build_failed\n",
        CreationOutcome::TimedOut => "timed_out\n",
    }
}

fn remove_if_there(directory: &Path) -> io::Result<()> {
    if fs::symlink_metadata(directory).is_ok() {
        fs::remove_dir_all(directory)
    } else {
        Ok(())
    }
}
