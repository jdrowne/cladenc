use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{self, Path, PathBuf};

use super::StartOptions;
use crate::benchmark::{self, Benchmark, BenchmarkError};
use crate::holdout;
use crate::journal::Settings;
use crate::model;
use crate::outputs_directory::holds_only_harness_files;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Refused {
    pub(super) option: &'static str,
    pub(super) accepted: &'static str,
    pub(super) reason: Option<String>,
}

impl Display for Refused {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match &self.reason {
            Some(reason) => write!(
                formatter,
                "{} takes {}: {reason}",
                self.option, self.accepted
            ),
            None => write!(formatter, "{} takes {}", self.option, self.accepted),
        }
    }
}

impl StartOptions {
    pub(super) fn settings(&self) -> Result<Settings, Refused> {
        let benchmarks = benchmarks(&self.benchmark)?;
        let seed = rand::random();
        let holdout_tasks = holdout_tasks(&benchmarks, self.holdout_percentage, seed)?;
        let corroboration_tasks = corroboration_tasks(
            &benchmarks,
            &holdout_tasks,
            self.holdout_corroboration_percentage,
            seed,
        )?;
        Ok(Settings {
            primogenitor: primogenitor(&self.primogenitor)?,
            model: self.model.clone(),
            model_url: self.model_url.clone(),
            model_key_file: self
                .model_key_file
                .as_deref()
                .map(model_key_file)
                .transpose()?,
            docker_image: self.docker_image.clone().unwrap_or_else(image_name),
            dockerfile: self.dockerfile.as_deref().map(dockerfile).transpose()?,
            docker_build_timeout: self.docker_build_timeout,
            environment_image_repository: environment_image_repository(),
            harness_build_command: self.harness_build_command.clone(),
            harness_run_command: self.harness_run_command.clone(),
            benchmarks,
            holdout_percentage: self.holdout_percentage,
            holdout_tasks,
            holdout_corroboration_percentage: self.holdout_corroboration_percentage,
            corroboration_tasks,
            max_procreations: self.max_procreations,
            max_concurrent_containers: self.max_concurrent_containers,
            procreation_timeout: self.procreation_timeout,
            attempt_timeout: self.attempt_timeout,
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

fn holdout_tasks(
    benchmarks: &[Benchmark],
    percentage: f64,
    seed: u64,
) -> Result<Vec<String>, Refused> {
    holdout::draw(benchmarks, percentage, seed).map_err(|error| Refused {
        option: "--holdout-percentage",
        accepted: "a number above 0 and below 100 that holds out at least one task of every \
                   benchmark and leaves at least one",
        reason: Some(error.to_string()),
    })
}

fn corroboration_tasks(
    benchmarks: &[Benchmark],
    holdout_tasks: &[String],
    percentage: f64,
    seed: u64,
) -> Result<Vec<String>, Refused> {
    holdout::draw_corroboration(benchmarks, holdout_tasks, percentage, seed).map_err(|error| {
        Refused {
            option: "--holdout-corroboration-percentage",
            accepted: "a number above 0 and below 100 that makes at least one holdout task of \
                       every benchmark a corroboration task and leaves at least one a selection \
                       task",
            reason: Some(error.to_string()),
        }
    })
}

fn primogenitor(given: &Path) -> Result<PathBuf, Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|directory| {
            directory.to_str().is_some()
                && directory.is_dir()
                && holds_only_harness_files(directory).unwrap_or(false)
        })
        .ok_or(Refused {
            option: "--primogenitor",
            accepted: "an existing directory that holds only regular files, directories and \
                       symbolic links",
            reason: None,
        })
}

fn model_key_file(given: &Path) -> Result<PathBuf, Refused> {
    path::absolute(given)
        .ok()
        .filter(|file| file.to_str().is_some() && model::read_key(file).is_some())
        .ok_or(Refused {
            option: "--model-key-file",
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

fn dockerfile(given: &Path) -> Result<PathBuf, Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|file| file.to_str().is_some() && file.is_file())
        .ok_or(Refused {
            option: "--dockerfile",
            accepted: "an existing file",
            reason: None,
        })
}

fn benchmarks(given: &[PathBuf]) -> Result<Vec<Benchmark>, Refused> {
    let mut benchmarks = given
        .iter()
        .map(|given| benchmark(given))
        .collect::<Result<Vec<Benchmark>, Refused>>()?;
    benchmarks.sort_by_key(Benchmark::name);
    benchmark::told_apart(&benchmarks)
        .map(|()| benchmarks)
        .map_err(|error| refused_benchmark(error.to_string()))
}

fn benchmark(given: &Path) -> Result<Benchmark, Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|directory| directory.to_str().is_some())
        .ok_or(BenchmarkError::NotADirectory)
        .and_then(|directory| Benchmark::read(&directory))
        .map_err(|error| refused_benchmark(format!("{}: {error}", given.display())))
}

fn refused_benchmark(reason: String) -> Refused {
    Refused {
        option: "--benchmark",
        accepted: "the directory of a benchmark, in which every directory is a task that holds \
                   a file of text named instruction.md, a directory named workspace, a directory \
                   named tests with an executable file named test.sh in it and a directory named \
                   environment with a file named Dockerfile and no Compose file in it",
        reason: Some(reason),
    }
}
