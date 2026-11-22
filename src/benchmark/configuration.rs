use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use super::artifacts::{self, Artifact, OF_THE_HARNESS};
use super::{is_name, valid};

pub(super) const CONFIGURATION: &str = "task.toml";
const VERIFIER: &str = "verifier";
const SEPARATE: &str = "separate";
const EVERYTHING: &str = "the whole container of the harness in the container of the grader";
const TIME_LIMIT_LEFT_OUT: Duration = Duration::from_secs(600);
const COLLECT_TIME_LIMIT: Duration = Duration::from_secs(60);
const ELSEWHERE: &str = "a command in a container other than that of the harness";
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

struct Keys {
    table: &'static str,
    build: &'static str,
    cpus: &'static str,
    memory: &'static str,
    megabytes: &'static str,
    check_time_limit: &'static str,
    check_interval: &'static str,
    check_start_period: &'static str,
    check_start_interval: &'static str,
    check_retries: &'static str,
}

macro_rules! keys {
    ($table:literal) => {
        Keys {
            table: $table,
            build: concat!($table, ".build_timeout_sec"),
            cpus: concat!($table, ".cpus"),
            memory: concat!($table, ".memory"),
            megabytes: concat!($table, ".memory_mb"),
            check_time_limit: concat!($table, ".healthcheck.timeout_sec"),
            check_interval: concat!($table, ".healthcheck.interval_sec"),
            check_start_period: concat!($table, ".healthcheck.start_period_sec"),
            check_start_interval: concat!($table, ".healthcheck.start_interval_sec"),
            check_retries: concat!($table, ".healthcheck.retries"),
        }
    };
}

const OF_THE_TASK: Keys = keys!("environment");
const OF_THE_GRADER: Keys = keys!("verifier.environment");

#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentConfiguration {
    pub docker_image: Option<String>,
    pub working_directory: Option<String>,
    pub variables: Vec<(String, String)>,
    pub build_time_limit: Duration,
    pub cpus: Option<f64>,
    pub memory_megabytes: Option<u64>,
    pub health_check: Option<HealthCheck>,
}

impl Default for EnvironmentConfiguration {
    fn default() -> Self {
        Self {
            docker_image: None,
            working_directory: None,
            variables: Vec::new(),
            build_time_limit: TIME_LIMIT_LEFT_OUT,
            cpus: None,
            memory_megabytes: None,
            health_check: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectCommand {
    pub command: String,
    pub user: Option<String>,
    pub time_limit: Duration,
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
    pub grader_environment: Option<EnvironmentConfiguration>,
    pub collect_commands: Vec<CollectCommand>,
    pub artifacts: Vec<Artifact>,
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
            grader_environment: None,
            collect_commands: Vec::new(),
            artifacts: Vec::new(),
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
    StepsBack {
        path: String,
    },
    NotAPlace {
        destination: String,
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
            Self::StepsBack { path } => write!(
                formatter,
                "lists the path `{path}` under artifacts, and a path that is kept holds no `..`"
            ),
            Self::NotAPlace { destination } => write!(
                formatter,
                "gives a path under artifacts the destination `{destination}`, and a \
                 destination names a file or a directory by a path that does not begin with \
                 `/` and holds no `..`"
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
    artifacts: Option<toml::Value>,
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
    #[serde(default)]
    collect: Vec<WrittenCollect>,
}

#[derive(Deserialize)]
struct WrittenCollect {
    command: String,
    service: Option<String>,
    timeout_sec: Option<f64>,
    user: Option<User>,
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
    let Written {
        environment,
        agent,
        verifier,
        steps,
        artifacts,
    } = written;
    let own = verifier
        .environment
        .map(|own| own.try_into::<WrittenEnvironment>().unwrap_or_default());
    let needed = needed(steps.as_ref(), &environment).or_else(|| own.as_ref().and_then(needed_in));
    if let Some(what) = needed {
        return Err(ConfigurationError::Needs { what });
    }
    let of_the_task = settings(&OF_THE_TASK, environment, set)?;
    let grader = time_limit("verifier.timeout_sec", verifier.timeout_sec)?;
    let grader_environment = match own {
        Some(own) => Some(settings(&OF_THE_GRADER, own, set)?),
        None => (verifier.environment_mode.as_deref() == Some(SEPARATE)).then(|| {
            EnvironmentConfiguration {
                docker_image: None,
                ..of_the_task.clone()
            }
        }),
    };
    let artifacts = artifacts::read(artifacts)?;
    if grader_environment.is_some() && artifacts.iter().any(|artifact| artifact.path() == "/") {
        return Err(ConfigurationError::Needs { what: EVERYTHING });
    }
    Ok(TaskConfiguration {
        docker_image: of_the_task.docker_image,
        working_directory: of_the_task.working_directory,
        variables: of_the_task.variables,
        build_time_limit: of_the_task.build_time_limit,
        cpus: of_the_task.cpus,
        memory_megabytes: of_the_task.memory_megabytes,
        health_check: of_the_task.health_check,
        harness_user: given(agent.user.map(User::named)),
        harness_time_limit: time_limit("agent.timeout_sec", agent.timeout_sec)?,
        grader_user: given(verifier.user.map(User::named)),
        grader_variables: variables(VERIFIER, verifier.env, set)?,
        grader_time_limit: grader.unwrap_or(TIME_LIMIT_LEFT_OUT),
        grader_environment,
        collect_commands: collect_commands(verifier.collect)?,
        artifacts,
    })
}

fn collect_commands(
    written: Vec<WrittenCollect>,
) -> Result<Vec<CollectCommand>, ConfigurationError> {
    written
        .into_iter()
        .filter(|written| !written.command.is_empty())
        .map(|written| {
            let elsewhere = written
                .service
                .is_some_and(|service| !service.is_empty() && service != OF_THE_HARNESS);
            if elsewhere {
                return Err(ConfigurationError::Needs { what: ELSEWHERE });
            }
            let time_limit = time_limit("verifier.collect.timeout_sec", written.timeout_sec)?;
            Ok(CollectCommand {
                command: written.command,
                user: given(written.user.map(User::named)),
                time_limit: time_limit.unwrap_or(COLLECT_TIME_LIMIT),
            })
        })
        .collect()
}

fn settings(
    keys: &Keys,
    written: WrittenEnvironment,
    set: &dyn Fn(&str) -> Option<String>,
) -> Result<EnvironmentConfiguration, ConfigurationError> {
    let build = time_limit(keys.build, written.build_timeout_sec)?;
    let memory = megabytes(keys, written.memory, written.memory_mb)?;
    Ok(EnvironmentConfiguration {
        docker_image: given(written.docker_image),
        working_directory: given(written.workdir),
        variables: variables(keys.table, written.env, set)?,
        build_time_limit: build.unwrap_or(TIME_LIMIT_LEFT_OUT),
        cpus: above_0(keys.cpus, written.cpus)?,
        memory_megabytes: memory,
        health_check: health_check(keys, written.healthcheck)?,
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

fn health_check(
    keys: &Keys,
    written: Option<toml::Value>,
) -> Result<Option<HealthCheck>, ConfigurationError> {
    let Some(written) = written
        .and_then(|written| written.try_into::<WrittenHealthCheck>().ok())
        .filter(|written| !written.command.is_empty())
    else {
        return Ok(None);
    };
    let time_limit = time_limit(keys.check_time_limit, written.timeout_sec)?;
    let interval = wait(keys.check_interval, written.interval_sec)?;
    let start_period = wait(keys.check_start_period, written.start_period_sec)?;
    let start_interval = wait(keys.check_start_interval, written.start_interval_sec)?;
    let retries = whole_above_0(keys.check_retries, written.retries)?;
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

fn needed(steps: Option<&toml::Value>, environment: &WrittenEnvironment) -> Option<&'static str> {
    holds_some(steps)
        .then_some("several steps")
        .or_else(|| needed_in(environment))
}

fn needed_in(environment: &WrittenEnvironment) -> Option<&'static str> {
    [
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
    keys: &Keys,
    older: Option<toml::Value>,
    newer: Option<i64>,
) -> Result<Option<u64>, ConfigurationError> {
    let older = size(keys.memory, older)?;
    let newer = whole_above_0(keys.megabytes, newer)?;
    match (older, newer) {
        (Some(older), Some(newer)) if older != newer => {
            Err(ConfigurationError::TwoSizes { key: keys.memory })
        }
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
