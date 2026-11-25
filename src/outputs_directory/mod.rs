mod copy;

use std::collections::BTreeSet;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::benchmark::{Benchmark, SPECIFICATION};
use crate::file_system::FileSystem;
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, HarnessId};

use copy::copy_tree;
pub(crate) use copy::{holds_only_harness_files, make_removable, same_tree};

#[derive(Clone, Debug)]
pub struct OutputsDirectory {
    files: Arc<dyn FileSystem>,
    path: PathBuf,
    holdout_tasks: BTreeSet<String>,
}

impl OutputsDirectory {
    pub fn at(files: Arc<dyn FileSystem>, path: &Path) -> Self {
        Self {
            files,
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

    pub fn create(
        files: Arc<dyn FileSystem>,
        path: &Path,
        primogenitor: &Path,
    ) -> io::Result<Self> {
        let outputs_directory = Self::at(files, path);
        outputs_directory
            .make(primogenitor)
            .map(|()| outputs_directory)
    }

    fn make(&self, primogenitor: &Path) -> io::Result<()> {
        let files = self.files();
        let path = &self.path;
        files
            .make_directory(path)
            .and_then(|()| files.make_directory(&path.join("harnesses")))
            .and_then(|()| files.make_directory(&path.join("procreations")))
            .and_then(|()| files.make_directory(&path.join("baselines")))
            .and_then(|()| files.make_directory(&path.join("attempts")))
            .and_then(|()| files.make_directory(&path.join("holdout")))
            .and_then(|()| files.make_directory(&path.join("running")))
            .and_then(|()| files.make_directory(&path.join("benchmarks")))
            .and_then(|()| files.resolved(primogenitor))
            .and_then(|resolved| copy_tree(files, &resolved, &self.harness(0)))
            .and_then(|()| files.make_directory(&self.attempts(0)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn files(&self) -> &dyn FileSystem {
        self.files.as_ref()
    }

    pub fn copy_benchmark(&self, benchmark: &Benchmark) -> io::Result<()> {
        let files = self.files();
        let kept = self.path.join("benchmarks").join(benchmark.name());
        files.make_directory(&kept).and_then(|()| {
            benchmark.tasks.iter().try_for_each(|id| {
                files
                    .resolved(&benchmark.directory.join(id))
                    .and_then(|resolved| copy_tree(files, &resolved, &kept.join(id)))
            })
        })
    }

    pub fn task(&self, task: &str) -> PathBuf {
        self.path.join("benchmarks").join(task)
    }

    pub fn image(&self) -> PathBuf {
        self.path.join("image")
    }

    pub(crate) fn start_image(&self, dockerfile: &Path) -> io::Result<PathBuf> {
        let image = self.image();
        self.files
            .make_directory(&image)
            .and_then(|()| self.files.make_directory(&image.join("context")))
            .and_then(|()| self.files.copy(dockerfile, &image.join("Dockerfile")))
            .map(|()| image)
    }

    pub fn environment(&self, task: &str) -> PathBuf {
        self.path.join("environments").join(task)
    }

    pub(crate) fn start_environment(&self, task: &str) -> io::Result<PathBuf> {
        let environment = self.environment(task);
        self.files
            .make_directories(&environment)
            .map(|()| environment)
    }

    pub fn verifier_environment(&self, task: &str) -> PathBuf {
        self.environment(task).join("verifier")
    }

    pub(crate) fn start_verifier_environment(&self, task: &str) -> io::Result<PathBuf> {
        let environment = self.verifier_environment(task);
        self.files
            .make_directories(&environment)
            .map(|()| environment)
    }

    pub fn harness(&self, id: HarnessId) -> PathBuf {
        self.path.join("harnesses").join(id.to_string())
    }

    pub fn procreation(&self, child: HarnessId) -> PathBuf {
        self.path.join("procreations").join(child.to_string())
    }

    pub(crate) fn baseline(&self, harness: HarnessId) -> PathBuf {
        self.path.join("baselines").join(harness.to_string())
    }

    pub fn attempts(&self, harness: HarnessId) -> PathBuf {
        self.path.join("attempts").join(harness.to_string())
    }

    fn holdout(&self, harness: HarnessId) -> PathBuf {
        self.path.join("holdout").join(harness.to_string())
    }

    pub fn attempt(&self, harness: HarnessId, task: &str) -> PathBuf {
        if self.holdout_tasks.contains(task) {
            self.holdout(harness).join(task)
        } else {
            self.attempts(harness).join(task)
        }
    }

    pub(crate) fn build(&self, harness: HarnessId) -> PathBuf {
        self.path.join("builds").join(harness.to_string())
    }

    pub fn running_procreation(&self, child: HarnessId) -> PathBuf {
        self.path
            .join("running")
            .join(format!("procreation-{child}"))
    }

    pub(crate) fn running_baseline(&self, harness: HarnessId) -> PathBuf {
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
        let files = self.files();
        files
            .make_directory(&running)
            .and_then(|()| copy_tree(files, &harness, &running.join("progenitor")))
            .and_then(|()| copy_tree(files, &harness, &running.join("child")))
    }

    pub fn end_procreation(
        &self,
        child: HarnessId,
        progenitor: HarnessId,
        outcome: CreationOutcome,
    ) -> io::Result<()> {
        let running = self.running_procreation(child);
        self.keep_made(&running.join("child"), child, outcome)
            .and_then(|()| self.files.remove_tree(&running.join("progenitor")))
            .and_then(|()| self.note(&running.join("progenitor"), &format!("{progenitor}\n")))
            .and_then(|()| self.note(&running.join("outcome"), word_of(outcome)))
            .and_then(|()| self.files.rename(&running, &self.procreation(child)))
    }

    pub(crate) fn start_baseline(
        &self,
        harness: HarnessId,
        progenitor: HarnessId,
    ) -> io::Result<()> {
        let running = self.running_baseline(harness);
        let files = self.files();
        files
            .make_directory(&running)
            .and_then(|()| copy_tree(files, &self.harness(0), &running.join("primogenitor")))
            .and_then(|()| {
                copy_tree(
                    files,
                    &self.harness(progenitor),
                    &running.join("singleton-control-group-harness"),
                )
            })
    }

    pub(crate) fn end_baseline(
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
        .and_then(|()| self.files.remove_tree(&running.join("primogenitor")))
        .and_then(|()| self.note(&running.join("progenitor"), &format!("{progenitor}\n")))
        .and_then(|()| self.note(&running.join("child"), &format!("{child}\n")))
        .and_then(|()| self.note(&running.join("outcome"), word_of(outcome)))
        .and_then(|()| self.files.rename(&running, &self.baseline(harness)))
    }

    fn note(&self, file: &Path, text: &str) -> io::Result<()> {
        self.files.write(file, text.as_bytes())
    }

    fn keep_made(
        &self,
        made: &Path,
        harness: HarnessId,
        outcome: CreationOutcome,
    ) -> io::Result<()> {
        self.files
            .rename(made, &self.harness(harness))
            .and_then(|()| match outcome {
                CreationOutcome::Created => self.files.make_directory(&self.attempts(harness)),
                CreationOutcome::Unchanged
                | CreationOutcome::Failed
                | CreationOutcome::BuildFailed
                | CreationOutcome::TimedOut => Ok(()),
            })
    }

    fn progenitor_of(&self, harness: HarnessId) -> Option<HarnessId> {
        self.files
            .read_text(&self.procreation(harness).join("progenitor"))
            .ok()
            .and_then(|id| id.trim().parse().ok())
    }

    pub(crate) fn line_of(&self, harness: HarnessId) -> Vec<HarnessId> {
        iter::successors(self.progenitor_of(harness), |ancestor| {
            self.progenitor_of(*ancestor)
                .filter(|earlier| earlier < ancestor)
        })
        .collect()
    }

    pub(crate) fn children_of(&self, progenitor: HarnessId) -> Vec<HarnessId> {
        let mut children: Vec<HarnessId> = self
            .files
            .list(&self.path.join("procreations"))
            .unwrap_or_default()
            .iter()
            .filter_map(|made| made.to_string_lossy().parse().ok())
            .filter(|child| self.progenitor_of(*child) == Some(progenitor))
            .collect();
        children.sort_unstable();
        children
    }

    pub fn start_build(&self, harness: HarnessId) -> io::Result<PathBuf> {
        let build = self.build(harness);
        remove_if_there(self.files(), &build)
            .and_then(|()| self.files.make_directories(&build))
            .map(|()| build)
    }

    pub fn start_attempt(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        let running = self.running_attempt(harness, task);
        let kept = self.attempt(harness, task);
        let files = self.files();
        files
            .list(&self.harness(harness))
            .and_then(|_| files.make_directory(&running))
            .and_then(|()| files.make_directories(&running.join("verifier/logs/verifier")))
            .and_then(|()| {
                files.copy(
                    &self.task(task).join(SPECIFICATION),
                    &running.join("specification"),
                )
            })
            .and_then(|()| {
                kept.parent()
                    .map_or(Ok(()), |above| files.make_directories(above))
            })
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
        self.note(&running.join("result"), word)
            .and_then(|()| self.files.rename(&running, &self.attempt(harness, task)))
    }

    pub(crate) fn discard_procreation(&self, child: HarnessId) -> io::Result<()> {
        [
            self.running_procreation(child),
            self.harness(child),
            self.procreation(child),
            self.attempts(child),
            self.build(child),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(self.files(), left))
    }

    pub(crate) fn discard_baseline(&self, harness: HarnessId) -> io::Result<()> {
        [
            self.running_baseline(harness),
            self.harness(harness),
            self.baseline(harness),
            self.attempts(harness),
            self.build(harness),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(self.files(), left))
    }

    pub(crate) fn discard_attempt(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        [
            self.running_attempt(harness, task),
            self.attempt(harness, task),
        ]
        .iter()
        .try_for_each(|left| remove_if_there(self.files(), left))
    }
}

fn word_of(outcome: CreationOutcome) -> &'static str {
    match outcome {
        CreationOutcome::Created => "created\n",
        CreationOutcome::Unchanged => "unchanged\n",
        CreationOutcome::Failed => "failed\n",
        CreationOutcome::BuildFailed => "build_failed\n",
        CreationOutcome::TimedOut => "timed_out\n",
    }
}

fn remove_if_there(files: &dyn FileSystem, directory: &Path) -> io::Result<()> {
    if files.found(directory).is_ok() {
        files.remove_tree(directory)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
