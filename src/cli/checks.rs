use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::RunOptions;
use crate::journal::Settings;
use crate::workspace::holds_only_harness_files;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refused {
    pub option: &'static str,
    pub accepted: &'static str,
}

impl Display for Refused {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} takes {}", self.option, self.accepted)
    }
}

impl RunOptions {
    pub fn settings(&self) -> Result<Settings, Refused> {
        Ok(Settings {
            progenitor: progenitor(&self.progenitor)?,
            tasks: tasks(&self.tasks)?,
            expander: program(&self.expander, "--expander")?,
            evaluator: program(&self.evaluator, "--evaluator")?,
            budget: self.budget,
            genealogy_growth: self.genealogy_growth,
            confidence: self.confidence,
            initial_expansions: self.initial_expansions,
            workers: self.workers,
            expansion_timeout: self.expansion_timeout,
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
        })
}

fn tasks(given: &Path) -> Result<Vec<String>, Refused> {
    fs::read_to_string(given)
        .ok()
        .map(|text| text.lines().map(str::to_owned).collect::<Vec<_>>())
        .filter(|ids| {
            !ids.is_empty()
                && ids.iter().all(|id| is_task_id(id))
                && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
        })
        .ok_or(Refused {
            option: "--tasks",
            accepted: "a file of task ids, one per line and at least one, each made of letters, \
                       digits, dots, underscores and hyphens, none of them . or .. and none \
                       repeated",
        })
}

fn is_task_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
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
        })
}
