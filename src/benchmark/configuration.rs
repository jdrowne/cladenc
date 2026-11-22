use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use super::is_name;

pub(super) const CONFIGURATION: &str = "task.toml";
const ENVIRONMENT: &str = "environment";
const VERIFIER: &str = "verifier";
const TIME_LIMIT_LEFT_OUT: Duration = Duration::from_secs(600);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskConfiguration {
    pub docker_image: Option<String>,
    pub working_directory: Option<String>,
    pub variables: Vec<(String, String)>,
    pub build_time_limit: Duration,
    pub harness_user: Option<String>,
    pub harness_time_limit: Option<Duration>,
    pub grader_user: Option<String>,
    pub grader_variables: Vec<(String, String)>,
    pub grader_time_limit: Duration,
}

impl Default for TaskConfiguration {
    fn default() -> Self {
        Self {
            docker_image: None,
            working_directory: None,
            variables: Vec::new(),
            build_time_limit: TIME_LIMIT_LEFT_OUT,
            harness_user: None,
            harness_time_limit: None,
            grader_user: None,
            grader_variables: Vec::new(),
            grader_time_limit: TIME_LIMIT_LEFT_OUT,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationError {
    Unreadable {
        reason: String,
    },
    NotValid {
        line: usize,
        reason: String,
    },
    NotAName {
        table: &'static str,
        name: String,
    },
    NotSet {
        table: &'static str,
        name: String,
        variable: String,
    },
    NotOneLine {
        table: &'static str,
        name: String,
    },
    NotATimeLimit {
        key: &'static str,
    },
}

impl Display for ConfigurationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { reason } => write!(formatter, "cannot be read: {reason}"),
            Self::NotValid { line, reason } => {
                write!(formatter, "is not valid at line {line}: {reason}")
            }
            Self::NotAName { table, name } => write!(
                formatter,
                "names a variable `{name}` under {table}.env, and the name of a variable is made \
                 of letters, digits, dots, underscores and hyphens alone"
            ),
            Self::NotSet {
                table,
                name,
                variable,
            } => write!(
                formatter,
                "gives {name} under {table}.env the value of {variable}, which is not set where \
                 cladenc runs"
            ),
            Self::NotOneLine { table, name } => write!(
                formatter,
                "gives {name} under {table}.env a value that is not one line of text, which a \
                 container cannot be given"
            ),
            Self::NotATimeLimit { key } => write!(
                formatter,
                "gives {key} a value that is not a number of seconds above 0"
            ),
        }
    }
}

#[derive(Default, Deserialize)]
struct Written {
    #[serde(default)]
    environment: WrittenEnvironment,
    #[serde(default)]
    agent: WrittenAgent,
    #[serde(default)]
    verifier: WrittenVerifier,
}

#[derive(Default, Deserialize)]
struct WrittenEnvironment {
    docker_image: Option<String>,
    workdir: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    build_timeout_sec: Option<f64>,
}

#[derive(Default, Deserialize)]
struct WrittenAgent {
    user: Option<User>,
    timeout_sec: Option<f64>,
}

#[derive(Default, Deserialize)]
struct WrittenVerifier {
    user: Option<User>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    timeout_sec: Option<f64>,
}

#[derive(Deserialize)]
#[serde(
    untagged,
    expecting = "invalid type: expected the name of a user, or the number of one, 0 or more"
)]
enum User {
    Name(String),
    Number(u32),
}

impl User {
    fn named(self) -> String {
        match self {
            Self::Name(name) => name,
            Self::Number(number) => number.to_string(),
        }
    }
}

impl TaskConfiguration {
    pub fn read(task: &Path) -> Result<Self, ConfigurationError> {
        read_with(task, &|variable| env::var(variable).ok())
    }
}

pub(super) fn read_with(
    task: &Path,
    set: &dyn Fn(&str) -> Option<String>,
) -> Result<TaskConfiguration, ConfigurationError> {
    let text =
        written_in(&task.join(CONFIGURATION)).map_err(|error| ConfigurationError::Unreadable {
            reason: error.to_string(),
        })?;
    let written: Written = toml::from_str(&text).map_err(|error| ConfigurationError::NotValid {
        line: line_of(&text, error.span().map_or(0, |span| span.start)),
        reason: error.message().to_owned(),
    })?;
    let build = time_limit(
        "environment.build_timeout_sec",
        written.environment.build_timeout_sec,
    )?;
    let grader = time_limit("verifier.timeout_sec", written.verifier.timeout_sec)?;
    Ok(TaskConfiguration {
        docker_image: given(written.environment.docker_image),
        working_directory: given(written.environment.workdir),
        variables: variables(ENVIRONMENT, written.environment.env, set)?,
        build_time_limit: build.unwrap_or(TIME_LIMIT_LEFT_OUT),
        harness_user: given(written.agent.user.map(User::named)),
        harness_time_limit: time_limit("agent.timeout_sec", written.agent.timeout_sec)?,
        grader_user: given(written.verifier.user.map(User::named)),
        grader_variables: variables(VERIFIER, written.verifier.env, set)?,
        grader_time_limit: grader.unwrap_or(TIME_LIMIT_LEFT_OUT),
    })
}

fn time_limit(
    key: &'static str,
    seconds: Option<f64>,
) -> Result<Option<Duration>, ConfigurationError> {
    seconds
        .map(|seconds| {
            Duration::try_from_secs_f64(seconds)
                .ok()
                .filter(|time_limit| !time_limit.is_zero())
                .ok_or(ConfigurationError::NotATimeLimit { key })
        })
        .transpose()
}

fn written_in(file: &Path) -> io::Result<String> {
    match fs::read_to_string(file) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        read => read,
    }
}

fn line_of(text: &str, place: usize) -> usize {
    text.get(..place).unwrap_or(text).matches('\n').count() + 1
}

fn given(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn variables(
    table: &'static str,
    written: BTreeMap<String, String>,
    set: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<(String, String)>, ConfigurationError> {
    written
        .into_iter()
        .map(|(name, value)| {
            if !is_name(&name) {
                return Err(ConfigurationError::NotAName { table, name });
            }
            match resolved(&value, set) {
                Err(variable) => Err(ConfigurationError::NotSet {
                    table,
                    name,
                    variable,
                }),
                Ok(value) if value.contains(['\n', '\r', '\0']) => {
                    Err(ConfigurationError::NotOneLine { table, name })
                }
                Ok(value) => Ok((name, value)),
            }
        })
        .collect()
}

fn resolved(written: &str, set: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    let mut resolved = String::new();
    let mut rest = written;
    while let Some((before, (named, after))) = rest
        .split_once("${")
        .and_then(|(before, template)| template.split_once('}').map(|parts| (before, parts)))
    {
        let (variable, default) = match named.split_once(":-") {
            Some((variable, default)) => (variable, Some(default)),
            None => (named, None),
        };
        let value = set(variable)
            .or_else(|| default.map(str::to_owned))
            .ok_or_else(|| variable.to_owned())?;
        resolved.push_str(before);
        resolved.push_str(&value);
        rest = after;
    }
    resolved.push_str(rest);
    Ok(resolved)
}
