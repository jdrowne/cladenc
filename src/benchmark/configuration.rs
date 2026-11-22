use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use super::{is_name, valid};

pub(super) const CONFIGURATION: &str = "task.toml";
const ENVIRONMENT: &str = "environment";
const VERIFIER: &str = "verifier";
const TIME_LIMIT_LEFT_OUT: Duration = Duration::from_secs(600);
const CHECKED_EVERY: Duration = Duration::from_secs(5);
const CHECK_TIME_LIMIT: Duration = Duration::from_secs(30);
const RETRIES_LEFT_OUT: u64 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthCheck {
    pub command: String,
    pub interval: Duration,
    pub time_limit: Duration,
    pub start_period: Duration,
    pub start_interval: Duration,
    pub retries: u64,
}

impl HealthCheck {
    pub(crate) fn longest(&self) -> Duration {
        let retries = u32::try_from(self.retries).unwrap_or(u32::MAX);
        self.time_limit
            .saturating_add(self.interval)
            .saturating_mul(retries)
            .saturating_add(self.time_limit)
            .saturating_add(self.start_period)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskConfiguration {
    pub docker_image: Option<String>,
    pub working_directory: Option<String>,
    pub variables: Vec<(String, String)>,
    pub build_time_limit: Duration,
    pub cpus: Option<f64>,
    pub memory_megabytes: Option<u64>,
    pub health_check: Option<HealthCheck>,
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
            cpus: None,
            memory_megabytes: None,
            health_check: None,
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
    Missing,
    Unreadable {
        reason: String,
    },
    NotValid {
        line: usize,
        reason: String,
    },
    SharedAndSeparate,
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
    NotAbove0 {
        key: &'static str,
    },
    NotAWait {
        key: &'static str,
    },
    NotASize {
        key: &'static str,
    },
    TwoSizes {
        key: &'static str,
    },
    Needs {
        what: &'static str,
    },
}

impl Display for ConfigurationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(formatter, "is not there"),
            Self::Unreadable { reason } => write!(formatter, "cannot be read: {reason}"),
            Self::NotValid { line, reason } => {
                write!(formatter, "is not valid at line {line}: {reason}")
            }
            Self::SharedAndSeparate => write!(
                formatter,
                "says under verifier that the grader shares the environment of the harness, as \
                 environment_mode, and gives the grader an environment of its own, as environment"
            ),
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
            Self::NotAbove0 { key } => write!(
                formatter,
                "gives {key} a value that is not a number above 0"
            ),
            Self::NotAWait { key } => write!(
                formatter,
                "gives {key} a value that is not a number of seconds, 0 or more"
            ),
            Self::NotASize { key } => write!(
                formatter,
                "gives {key} a value that is not a size of a megabyte or more, written as a \
                 number and one of G, M and K, such as 4G"
            ),
            Self::TwoSizes { key } => {
                write!(formatter, "gives {key} and {key}_mb sizes that differ")
            }
            Self::Needs { what } => write!(
                formatter,
                "asks for {what}, which cladenc has no way to give"
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
    steps: Option<toml::Value>,
}

#[derive(Default, Deserialize)]
struct WrittenEnvironment {
    docker_image: Option<String>,
    workdir: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    build_timeout_sec: Option<f64>,
    cpus: Option<f64>,
    memory_mb: Option<i64>,
    memory: Option<toml::Value>,
    healthcheck: Option<toml::Value>,
    gpus: Option<i64>,
    tpu: Option<toml::Value>,
    os: Option<String>,
    mcp_servers: Option<toml::Value>,
    skills_dir: Option<String>,
}

#[derive(Deserialize)]
struct WrittenHealthCheck {
    command: String,
    interval_sec: Option<f64>,
    timeout_sec: Option<f64>,
    start_period_sec: Option<f64>,
    start_interval_sec: Option<f64>,
    retries: Option<i64>,
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
    environment_mode: Option<String>,
    environment: Option<toml::Value>,
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
    let text = written_in(&task.join(CONFIGURATION))?;
    let written = toml::from_str::<Written>(&text)
        .and_then(|written| valid::check(&text).map(|()| written))
        .map_err(|error| ConfigurationError::NotValid {
            line: line_of(&text, error.span().map_or(0, |span| span.start)),
            reason: error.message().to_owned(),
        })?;
    if shares_and_has_its_own(&written.verifier) {
        return Err(ConfigurationError::SharedAndSeparate);
    }
    if let Some(what) = needed(&written) {
        return Err(ConfigurationError::Needs { what });
    }
    let build = time_limit(
        "environment.build_timeout_sec",
        written.environment.build_timeout_sec,
    )?;
    let grader = time_limit("verifier.timeout_sec", written.verifier.timeout_sec)?;
    let memory = megabytes(
        "environment.memory",
        written.environment.memory,
        written.environment.memory_mb,
    )?;
    Ok(TaskConfiguration {
        docker_image: given(written.environment.docker_image),
        working_directory: given(written.environment.workdir),
        variables: variables(ENVIRONMENT, written.environment.env, set)?,
        build_time_limit: build.unwrap_or(TIME_LIMIT_LEFT_OUT),
        cpus: above_0("environment.cpus", written.environment.cpus)?,
        memory_megabytes: memory,
        health_check: health_check(written.environment.healthcheck)?,
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

fn health_check(written: Option<toml::Value>) -> Result<Option<HealthCheck>, ConfigurationError> {
    let Some(written) = written
        .and_then(|written| written.try_into::<WrittenHealthCheck>().ok())
        .filter(|written| !written.command.is_empty())
    else {
        return Ok(None);
    };
    let time_limit = time_limit("environment.healthcheck.timeout_sec", written.timeout_sec)?;
    let interval = wait("environment.healthcheck.interval_sec", written.interval_sec)?;
    let start_period = wait(
        "environment.healthcheck.start_period_sec",
        written.start_period_sec,
    )?;
    let start_interval = wait(
        "environment.healthcheck.start_interval_sec",
        written.start_interval_sec,
    )?;
    let retries = whole_above_0("environment.healthcheck.retries", written.retries)?;
    Ok(Some(HealthCheck {
        command: written.command,
        interval: interval.unwrap_or(CHECKED_EVERY),
        time_limit: time_limit.unwrap_or(CHECK_TIME_LIMIT),
        start_period: start_period.unwrap_or_default(),
        start_interval: start_interval.unwrap_or(CHECKED_EVERY),
        retries: retries.unwrap_or(RETRIES_LEFT_OUT),
    }))
}

fn wait(key: &'static str, seconds: Option<f64>) -> Result<Option<Duration>, ConfigurationError> {
    seconds
        .map(|seconds| {
            Duration::try_from_secs_f64(seconds).map_err(|_| ConfigurationError::NotAWait { key })
        })
        .transpose()
}

fn whole_above_0(
    key: &'static str,
    number: Option<i64>,
) -> Result<Option<u64>, ConfigurationError> {
    number
        .map(|number| {
            u64::try_from(number)
                .ok()
                .filter(|number| *number > 0)
                .ok_or(ConfigurationError::NotAbove0 { key })
        })
        .transpose()
}

fn shares_and_has_its_own(verifier: &WrittenVerifier) -> bool {
    verifier.environment_mode.as_deref() == Some("shared") && verifier.environment.is_some()
}

fn needed(written: &Written) -> Option<&'static str> {
    let environment = &written.environment;
    [
        (holds_some(written.steps.as_ref()), "several steps"),
        (environment.gpus.is_some_and(|gpus| gpus > 0), "a GPU"),
        (environment.tpu.is_some(), "a TPU"),
        (environment.os.as_deref() == Some("windows"), "Windows"),
        (holds_some(environment.mcp_servers.as_ref()), "MCP servers"),
        (
            environment
                .skills_dir
                .as_deref()
                .is_some_and(|directory| !directory.is_empty()),
            "a skills directory",
        ),
    ]
    .into_iter()
    .find_map(|(needed, what)| needed.then_some(what))
}

fn holds_some(list: Option<&toml::Value>) -> bool {
    list.and_then(toml::Value::as_array)
        .is_some_and(|list| !list.is_empty())
}

fn above_0(key: &'static str, number: Option<f64>) -> Result<Option<f64>, ConfigurationError> {
    number
        .map(|number| {
            if number.is_finite() && number > 0.0 {
                Ok(number)
            } else {
                Err(ConfigurationError::NotAbove0 { key })
            }
        })
        .transpose()
}

fn megabytes(
    key: &'static str,
    older: Option<toml::Value>,
    newer: Option<i64>,
) -> Result<Option<u64>, ConfigurationError> {
    let older = size(key, older)?;
    let newer = whole_above_0("environment.memory_mb", newer)?;
    match (older, newer) {
        (Some(older), Some(newer)) if older != newer => Err(ConfigurationError::TwoSizes { key }),
        (older, newer) => Ok(newer.or(older)),
    }
}

fn size(
    key: &'static str,
    written: Option<toml::Value>,
) -> Result<Option<u64>, ConfigurationError> {
    let Some(toml::Value::String(written)) = written else {
        return Ok(None);
    };
    let written = written.trim().to_uppercase();
    [('G', 1024.0), ('M', 1.0), ('K', 1.0 / 1024.0)]
        .into_iter()
        .find_map(|(unit, megabytes)| {
            written
                .strip_suffix(unit)
                .and_then(|number| number.parse::<f64>().ok())
                .map(|number| number * megabytes)
        })
        .filter(|megabytes| megabytes.is_finite() && *megabytes >= 1.0)
        .map(|megabytes| Some(megabytes as u64))
        .ok_or(ConfigurationError::NotASize { key })
}

fn written_in(file: &Path) -> Result<String, ConfigurationError> {
    fs::read_to_string(file).map_err(|error| match error.kind() {
        ErrorKind::NotFound => ConfigurationError::Missing,
        _ => ConfigurationError::Unreadable {
            reason: error.to_string(),
        },
    })
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
