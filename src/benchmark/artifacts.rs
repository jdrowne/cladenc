use serde::Deserialize;

use super::configuration::ConfigurationError;

pub(super) const OF_THE_HARNESS: &str = "main";
pub(crate) const PUBLISHED: &str = "/logs/artifacts";
const STEP_BACK: &str = "..";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    pub(crate) source: String,
    pub(crate) destination: String,
    pub(crate) exclude: Vec<String>,
}

impl Artifact {
    pub(crate) fn path(&self) -> String {
        format!("/{}", parts(&self.source).join("/"))
    }

    pub(crate) fn published(listed: &[Self]) -> Option<Self> {
        let published = Self {
            source: PUBLISHED.to_owned(),
            destination: parts(PUBLISHED).join("/"),
            exclude: Vec::new(),
        };
        let is_listed = listed
            .iter()
            .any(|artifact| artifact.path() == PUBLISHED || overlap(artifact, &published));
        (!is_listed).then_some(published)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Written {
    Path(String),
    Table {
        source: String,
        destination: Option<String>,
        #[serde(default)]
        exclude: Vec<String>,
        service: Option<String>,
    },
}

pub(super) fn read(written: Option<toml::Value>) -> Result<Vec<Artifact>, ConfigurationError> {
    let written: Vec<Written> = written
        .and_then(|written| written.try_into().ok())
        .unwrap_or_default();
    written.into_iter().try_fold(Vec::new(), |mut kept, entry| {
        let artifact = artifact(entry)?;
        if !kept.iter().any(|earlier| overlap(earlier, &artifact)) {
            kept.push(artifact);
        }
        Ok(kept)
    })
}

fn artifact(written: Written) -> Result<Artifact, ConfigurationError> {
    let (source, destination, exclude, service) = match written {
        Written::Path(source) => (source, None, Vec::new(), None),
        Written::Table {
            source,
            destination,
            exclude,
            service,
        } => (source, destination, exclude, service),
    };
    if service.is_some_and(|service| !service.is_empty() && service != OF_THE_HARNESS) {
        return Err(ConfigurationError::Needs {
            what: "a path of a container other than that of the harness",
        });
    }
    if parts(&source).contains(&STEP_BACK) {
        return Err(ConfigurationError::StepsBack { path: source });
    }
    let destination = match destination.filter(|destination| !destination.is_empty()) {
        Some(destination) => {
            place(&destination).ok_or(ConfigurationError::NotAPlace { destination })?
        }
        None => parts(&source).join("/"),
    };
    Ok(Artifact {
        source,
        destination,
        exclude,
    })
}

fn place(destination: &str) -> Option<String> {
    let parts = parts(destination);
    let below = !destination.starts_with('/') && !parts.is_empty() && !parts.contains(&STEP_BACK);
    below.then(|| parts.join("/"))
}

fn parts(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect()
}

fn overlap(one: &Artifact, other: &Artifact) -> bool {
    parts(&one.destination)
        .iter()
        .zip(parts(&other.destination))
        .all(|(part, other)| *part == other)
}
