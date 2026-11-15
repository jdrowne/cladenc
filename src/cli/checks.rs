use std::fmt::{self, Display, Formatter};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{self, Path, PathBuf};

use super::StartOptions;
use crate::benchmark::{self, BenchmarkError};
use crate::journal::Settings;
use crate::model;
use crate::workspace::holds_only_harness_files;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refused {
    pub option: &'static str,
    pub accepted: &'static str,
    pub reason: Option<String>,
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
    pub fn settings(&self) -> Result<Settings, Refused> {
        let (benchmark, tasks) = benchmark(&self.benchmark)?;
        Ok(Settings {
            progenitor: progenitor(&self.progenitor)?,
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
            harness_build_command: self.harness_build_command.clone(),
            harness_run_command: self.harness_run_command.clone(),
            benchmark,
            tasks,
            evaluator: program(&self.evaluator, "--evaluator")?,
            budget: self.budget,
            genealogy_growth: self.genealogy_growth,
            confidence: self.confidence,
            initial_procreations: self.initial_procreations,
            workers: self.workers,
            procreation_timeout: self.procreation_timeout,
            evaluation_timeout: self.evaluation_timeout,
            seed: self.seed,
        })
    }
}

pub(super) fn genealogy_growth(given: &str) -> Result<f64, String> {
    given
        .parse()
        .ok()
        .filter(|growth| (0.0..=1.0).contains(growth))
        .ok_or_else(|| "a number from 0 to 1 is expected".to_owned())
}

pub(super) fn confidence(given: &str) -> Result<f64, String> {
    given
        .parse()
        .ok()
        .filter(|confidence| *confidence > 0.0 && *confidence < 100.0)
        .ok_or_else(|| "a number above 0 and below 100 is expected".to_owned())
}

fn progenitor(given: &Path) -> Result<PathBuf, Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|directory| {
            directory.to_str().is_some()
                && directory.is_dir()
                && holds_only_harness_files(directory).unwrap_or(false)
        })
        .ok_or(Refused {
            option: "--progenitor",
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

fn benchmark(given: &Path) -> Result<(PathBuf, Vec<String>), Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|directory| directory.to_str().is_some())
        .ok_or(BenchmarkError::NotADirectory)
        .and_then(|directory| benchmark::tasks(&directory).map(|tasks| (directory, tasks)))
        .map_err(|error| Refused {
            option: "--benchmark",
            accepted: "the directory of a benchmark, in which every directory is a task that \
                       holds a file of text named statement, a directory named files and an \
                       executable file named check",
            reason: Some(error.to_string()),
        })
}

fn program(given: &Path, option: &'static str) -> Result<PathBuf, Refused> {
    fs::canonicalize(given)
        .ok()
        .filter(|file| {
            file.to_str().is_some()
                && fs::metadata(file).is_ok_and(|metadata| {
                    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
                })
        })
        .ok_or(Refused {
            option,
            accepted: "an executable file",
            reason: None,
        })
}
