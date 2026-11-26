use std::collections::BTreeMap;
use std::fmt::{self, Formatter};

use serde::Deserialize;
use serde::de::{
    DeserializeSeed, Deserializer, Error, IgnoredAny, MapAccess, SeqAccess, Unexpected, Visitor,
};

#[derive(Clone, Copy)]
enum Kind {
    Anything,
    Text,
    Number,
    WholeNumber,
    Truth,
    OneOf(&'static [&'static str]),
    TextByName,
    List(&'static Kind),
    Table(&'static Table),
    PathOr(&'static Table),
}

struct Table {
    needed: &'static [&'static str],
    keys: &'static [(&'static str, Kind)],
}

const TEXTS: Kind = Kind::List(&Kind::Text);
const NETWORK_MODE: Kind = Kind::OneOf(&["no-network", "public", "allowlist"]);

static TASK_TOML: Table = Table {
    needed: &[],
    keys: &[
        ("schema_version", Kind::Text),
        (
            "multi_step_reward_strategy",
            Kind::OneOf(&["mean", "final"]),
        ),
        ("source", Kind::Text),
        ("task", Kind::Table(&TASK)),
        ("metadata", Kind::Table(&ANYTHING)),
        ("verifier", Kind::Table(&VERIFIER)),
        ("agent", Kind::Table(&AGENT)),
        ("solution", Kind::Table(&SOLUTION)),
        ("environment", Kind::Table(&ENVIRONMENT)),
        ("artifacts", Kind::List(&Kind::PathOr(&ARTIFACT))),
    ],
};

static TASK: Table = Table {
    needed: &["name"],
    keys: &[
        ("name", Kind::Text),
        ("authors", Kind::List(&Kind::Table(&AUTHOR))),
        ("description", Kind::Text),
        ("keywords", TEXTS),
    ],
};

static AUTHOR: Table = Table {
    needed: &["name"],
    keys: &[("name", Kind::Text), ("email", Kind::Text)],
};

static ANYTHING: Table = Table {
    needed: &[],
    keys: &[],
};

static VERIFIER: Table = Table {
    needed: &[],
    keys: &[
        ("network_mode", NETWORK_MODE),
        ("allowed_hosts", TEXTS),
        ("environment_mode", Kind::OneOf(&["shared", "separate"])),
        ("environment", Kind::Table(&ENVIRONMENT)),
    ],
};

static AGENT: Table = Table {
    needed: &[],
    keys: &[("network_mode", NETWORK_MODE), ("allowed_hosts", TEXTS)],
};

static SOLUTION: Table = Table {
    needed: &[],
    keys: &[("env", Kind::TextByName)],
};

static ENVIRONMENT: Table = Table {
    needed: &[],
    keys: &[
        ("build_timeout_sec", Kind::Number),
        ("network_mode", NETWORK_MODE),
        ("allowed_hosts", TEXTS),
        ("docker_image", Kind::Text),
        ("os", Kind::OneOf(&["linux", "windows"])),
        ("cpus", Kind::Number),
        ("memory_mb", Kind::WholeNumber),
        ("storage_mb", Kind::WholeNumber),
        ("gpus", Kind::WholeNumber),
        ("gpu_types", TEXTS),
        ("tpu", Kind::Table(&TPU)),
        ("allow_internet", Kind::Truth),
        ("env", Kind::TextByName),
        ("workdir", Kind::Text),
        ("skills_dir", Kind::Text),
        ("healthcheck", Kind::Table(&HEALTHCHECK)),
        ("mcp_servers", Kind::List(&Kind::Table(&MCP_SERVER))),
    ],
};

static TPU: Table = Table {
    needed: &["type", "topology"],
    keys: &[("type", Kind::Text), ("topology", Kind::Text)],
};

static HEALTHCHECK: Table = Table {
    needed: &["command"],
    keys: &[
        ("command", Kind::Text),
        ("interval_sec", Kind::Number),
        ("timeout_sec", Kind::Number),
        ("start_period_sec", Kind::Number),
        ("start_interval_sec", Kind::Number),
        ("retries", Kind::WholeNumber),
    ],
};

static MCP_SERVER: Table = Table {
    needed: &["name"],
    keys: &[
        ("name", Kind::Text),
        (
            "transport",
            Kind::OneOf(&["stdio", "sse", "streamable-http", "http"]),
        ),
        ("url", Kind::Text),
        ("headers", Kind::TextByName),
        ("command", Kind::Text),
        ("args", TEXTS),
    ],
};

static ARTIFACT: Table = Table {
    needed: &["source"],
    keys: &[
        ("source", Kind::Text),
        ("destination", Kind::Text),
        ("exclude", TEXTS),
        ("service", Kind::Text),
    ],
};

pub(super) fn check(written: &str) -> Result<(), toml::de::Error> {
    toml::de::Deserializer::parse(written)
        .and_then(|written| written.deserialize_map(Fields(&TASK_TOML)))
}

impl<'de> DeserializeSeed<'de> for Kind {
    type Value = ();

    fn deserialize<D>(self, written: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        match self {
            Self::Anything => is::<IgnoredAny, D>(written),
            Self::Text => is::<String, D>(written),
            Self::Number => is::<f64, D>(written),
            Self::WholeNumber => is::<i64, D>(written),
            Self::Truth => is::<bool, D>(written),
            Self::OneOf(allowed) => one_of(allowed, written),
            Self::TextByName => is::<BTreeMap<String, String>, D>(written),
            Self::List(kind) => written.deserialize_seq(Listed(kind)),
            Self::Table(table) => written.deserialize_map(Fields(table)),
            Self::PathOr(table) => written.deserialize_any(PathOr(table)),
        }
    }
}

fn is<'de, T, D>(written: D) -> Result<(), D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(written).map(|_| ())
}

fn one_of<'de, D>(allowed: &'static [&'static str], written: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    let written = String::deserialize(written)?;
    if allowed.contains(&written.as_str()) {
        Ok(())
    } else {
        let expected = format!("one of {}", allowed.join(", "));
        Err(D::Error::invalid_value(
            Unexpected::Str(&written),
            &expected.as_str(),
        ))
    }
}

struct Fields(&'static Table);

impl<'de> Visitor<'de> for Fields {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a table")
    }

    fn visit_map<A>(self, mut written: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut held = Vec::new();
        while let Ok(Some(name)) = written.next_key::<String>() {
            let kind = self
                .0
                .keys
                .iter()
                .find(|(key, _)| *key == name)
                .map_or(Kind::Anything, |(_, kind)| *kind);
            written.next_value_seed(kind)?;
            held.push(name);
        }
        let missing = self
            .0
            .needed
            .iter()
            .find(|key| !held.iter().any(|name| name == *key));
        match missing {
            Some(key) => Err(A::Error::missing_field(key)),
            None => Ok(()),
        }
    }
}

struct Listed(&'static Kind);

impl<'de> Visitor<'de> for Listed {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a list")
    }

    fn visit_seq<A>(self, mut written: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
    {
        while written.next_element_seed(*self.0)?.is_some() {}
        Ok(())
    }
}

struct PathOr(&'static Table);

impl<'de> Visitor<'de> for PathOr {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a path, or a table that names one as source")
    }

    fn visit_str<E>(self, _path: &str) -> Result<(), E>
    where
        E: Error,
    {
        Ok(())
    }

    fn visit_map<A>(self, written: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
    {
        Fields(self.0).visit_map(written)
    }
}
