mod copy;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, HarnessId, ProcreationOutcome};

pub use copy::{copy_tree, holds_only_harness_files};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunDirectory {
    path: PathBuf,
}

impl RunDirectory {
    pub fn at(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }

    pub fn create(path: &Path, primogenitor: &Path) -> io::Result<Self> {
        let run_directory = Self::at(path);
        fs::create_dir(path)
            .and_then(|()| fs::create_dir(path.join("harnesses")))
            .and_then(|()| fs::create_dir(path.join("procreations")))
            .and_then(|()| fs::create_dir(path.join("attempts")))
            .and_then(|()| fs::create_dir(path.join("running")))
            .and_then(|()| fs::create_dir(path.join("benchmarks")))
            .and_then(|()| fs::canonicalize(primogenitor))
            .and_then(|resolved| copy_tree(&resolved, &run_directory.harness(0)))
            .and_then(|()| fs::create_dir(run_directory.attempts(0)))
            .map(|()| run_directory)
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

    pub fn attempts(&self, harness: HarnessId) -> PathBuf {
        self.path.join("attempts").join(harness.to_string())
    }

    pub fn attempt(&self, harness: HarnessId, task: &str) -> PathBuf {
        self.attempts(harness).join(task)
    }

    pub fn build(&self, harness: HarnessId) -> PathBuf {
        self.path.join("builds").join(harness.to_string())
    }

    pub fn running_procreation(&self, child: HarnessId) -> PathBuf {
        self.path
            .join("running")
            .join(format!("procreation-{child}"))
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

    pub fn end_procreation(&self, child: HarnessId, outcome: ProcreationOutcome) -> io::Result<()> {
        let running = self.running_procreation(child);
        fs::rename(running.join("child"), self.harness(child))
            .and_then(|()| match outcome {
                ProcreationOutcome::Created => fs::create_dir(self.attempts(child)),
                ProcreationOutcome::Failed
                | ProcreationOutcome::BuildFailed
                | ProcreationOutcome::TimedOut => Ok(()),
            })
            .and_then(|()| fs::remove_dir_all(running.join("progenitor")))
            .and_then(|()| fs::rename(&running, self.procreation(child)))
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

    pub fn discard_attempt(&self, harness: HarnessId, task: &str) -> io::Result<()> {
        [
            self.running_attempt(harness, task),
            self.attempt(harness, task),
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
