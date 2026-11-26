use std::fmt::{self, Display, Formatter};
use std::path::{self, Path, PathBuf};

use super::StartOptions;
use crate::benchmark::{self, Benchmark, BenchmarkError};
use crate::file_system::{FileSystem, Kind};
use crate::holdout::{self, Drawn, HoldoutError, Percentages};
use crate::journal::Settings;
use crate::model;
use crate::outputs_directory::holds_only_harness_files;

const BENCHMARK: &[&str] = &["--benchmark"];
const SELECTION: &[&str] = &["--selection-tasks-percentage"];
const CORROBORATION: &[&str] = &["--corroboration-tasks-percentage"];
const SELECTION_AND_CORROBORATION: &[&str] = &[
    "--selection-tasks-percentage",
    "--corroboration-tasks-percentage",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Refused {
    pub(super) flags: &'static [&'static str],
    pub(super) accepted: &'static str,
    pub(super) reason: Option<String>,
}

impl Display for Refused {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let takes = if self.flags.len() == 1 {
            "takes"
        } else {
            "take"
        };
        let reason = self
            .reason
            .as_ref()
            .map_or_else(String::new, |reason| format!(": {reason}"));
        write!(
            formatter,
            "{} {takes} {}{reason}",
            self.flags.join(" and "),
            self.accepted
        )
    }
}

impl StartOptions {
    pub(super) fn settings(&self, files: &dyn FileSystem) -> Result<Settings, Refused> {
        let benchmarks = benchmarks(files, &self.benchmark)?;
        let seed = rand::random();
        let drawn = drawn(
            &benchmarks,
            Percentages {
                selection: self.selection_tasks_percentage,
                corroboration: self.corroboration_tasks_percentage,
            },
            seed,
        )?;
        Ok(Settings {
            primogenitor: primogenitor(files, &self.primogenitor)?,
            model: self.model.clone(),
            model_url: self.model_url.clone(),
            model_key_file: self
                .model_key_file
                .as_deref()
                .map(|given| model_key_file(files, given))
                .transpose()?,
            docker_image: self
                .diversification_docker_image
                .clone()
                .unwrap_or_else(image_name),
            dockerfile: self
                .diversification_dockerfile
                .as_deref()
                .map(|given| dockerfile(files, given))
                .transpose()?,
            docker_build_timeout: self.diversification_docker_image_build_timeout,
            environment_image_repository: environment_image_repository(),
            harness_build_command: self.harness_build_command.clone(),
            harness_run_command: self.harness_run_command.clone(),
            benchmarks,
            selection_tasks_percentage: self.selection_tasks_percentage,
            holdout_tasks: drawn.holdout_tasks,
            corroboration_tasks_percentage: self.corroboration_tasks_percentage,
            corroboration_tasks: drawn.corroboration_tasks,
            max_diversifications: self.max_diversifications,
            max_concurrent_containers: self.max_concurrent_containers,
            diversification_timeout: self.diversification_timeout,
            max_verifier_health_check_retries: self.max_verifier_health_check_retries,
            max_task_health_check_retries: self.max_task_health_check_retries,
            seed,
        })
    }
}

pub(super) fn percentage(given: &str) -> Result<f64, String> {
    given
        .parse()
        .ok()
        .filter(|percentage| 0.0 < *percentage && *percentage < 100.0)
        .ok_or_else(|| "a number above 0 and below 100 is expected".to_owned())
}

fn drawn(benchmarks: &[Benchmark], percentages: Percentages, seed: u64) -> Result<Drawn, Refused> {
    holdout::draw(benchmarks, percentages, seed).map_err(|error| {
        let (flags, accepted) = refused_for(&error);
        Refused {
            flags,
            accepted,
            reason: Some(error.to_string()),
        }
    })
}

fn refused_for(error: &HoldoutError) -> (&'static [&'static str], &'static str) {
    match error {
        HoldoutError::TooFewTasks { .. } => (
            BENCHMARK,
            "the directory of a benchmark of three tasks at the least",
        ),
        HoldoutError::NoSelectionTask { .. } => (
            SELECTION,
            "a number above 0 and below 100 that makes at least one task of every benchmark a \
             selection task",
        ),
        HoldoutError::NoCorroborationTask { .. } => (
            CORROBORATION,
            "a number above 0 and below 100 that makes at least one task of every benchmark a \
             corroboration task",
        ),
        HoldoutError::NoNonHoldoutTask { .. } => (
            SELECTION_AND_CORROBORATION,
            "numbers that together leave at least one task of every benchmark a non-holdout task",
        ),
    }
}

fn primogenitor(files: &dyn FileSystem, given: &Path) -> Result<PathBuf, Refused> {
    files
        .resolved(given)
        .ok()
        .filter(|directory| {
            directory.to_str().is_some()
                && leads_to(files, directory, Kind::Directory)
                && holds_only_harness_files(files, directory).unwrap_or(false)
        })
        .ok_or(Refused {
            flags: &["--primogenitor"],
            accepted: "an existing directory that holds only regular files, directories and \
                       symbolic links",
            reason: None,
        })
}

fn leads_to(files: &dyn FileSystem, path: &Path, kind: Kind) -> bool {
    files.followed(path).is_ok_and(|found| found.kind == kind)
}

fn model_key_file(files: &dyn FileSystem, given: &Path) -> Result<PathBuf, Refused> {
    path::absolute(given)
        .ok()
        .filter(|file| file.to_str().is_some() && model::read_key(files, file).is_some())
        .ok_or(Refused {
            flags: &["--model-key-file"],
            accepted: "a file that holds a key on one line and nothing else",
            reason: None,
        })
}

fn image_name() -> String {
    format!("cladenc-{:016x}", rand::random::<u64>())
}

fn environment_image_repository() -> String {
    format!("cladenc-environments-{:016x}", rand::random::<u64>())
}

fn dockerfile(files: &dyn FileSystem, given: &Path) -> Result<PathBuf, Refused> {
    files
        .resolved(given)
        .ok()
        .filter(|file| file.to_str().is_some() && leads_to(files, file, Kind::File))
        .ok_or(Refused {
            flags: &["--diversification-dockerfile"],
            accepted: "an existing file",
            reason: None,
        })
}

fn benchmarks(files: &dyn FileSystem, given: &[PathBuf]) -> Result<Vec<Benchmark>, Refused> {
    let mut benchmarks = given
        .iter()
        .map(|given| benchmark(files, given))
        .collect::<Result<Vec<Benchmark>, Refused>>()?;
    benchmarks.sort_by_key(Benchmark::name);
    benchmark::told_apart(&benchmarks)
        .map(|()| benchmarks)
        .map_err(|error| refused_benchmark(error.to_string()))
}

fn benchmark(files: &dyn FileSystem, given: &Path) -> Result<Benchmark, Refused> {
    files
        .resolved(given)
        .ok()
        .filter(|directory| directory.to_str().is_some())
        .ok_or(BenchmarkError::NotADirectory)
        .and_then(|directory| Benchmark::read(files, &directory))
        .map_err(|error| refused_benchmark(format!("{}: {error}", given.display())))
}

fn refused_benchmark(reason: String) -> Refused {
    Refused {
        flags: BENCHMARK,
        accepted: "the directory of a benchmark, in which every directory is a task that holds \
                   a file of text named instruction.md, a file named task.toml that is valid as \
                   the task configuration of Harbor, a directory named tests with an executable \
                   file named test.sh in it, or a verifier in an image of its own, and a directory \
                   named environment with a file named Dockerfile and no Compose file in it, or, \
                   in place of the Dockerfile, an image named in its task.toml",
        reason: Some(reason),
    }
}
