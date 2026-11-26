use std::ffi::OsString;
use std::fmt::Write;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use nix::unistd::{Gid, Uid};

use super::docker::{Docker, Host};
use super::left;
use super::program::{Ending, FOREVER, LEEWAY, Limits};
use super::{Copied, Environment, SandboxError};
use crate::benchmark::{HealthCheck, TaskConfiguration};
use crate::file_system::FileSystem;
use crate::outputs_directory::{OutputsDirectory, make_removable};

const NOT_STARTED: i32 = 125;
const MACHINE: &str = "host.docker.internal:host-gateway";
const TIMER: &str =
    r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) > /dev/null 2>&1 & wait $run"#;
const SETTINGS: &str = "{{println .Config.User}}{{.Config.WorkingDir}}";
const ROOT: &str = "0";
const STANDARD_INPUT: &str = "/dev/stdin";
const EMPTIED: &str = r#"rm -rf "$1" && mkdir -p "$1""#;
const ROOM_MADE: &str = r#"rm -rf "$1" && mkdir -p "${1%/*}/""#;
const END_EVERYTHING: &str = "kill -9 -1";
const COPYING: Duration = Duration::from_secs(600);
const FOLLOW_LINK: &str = "--follow-link";
const ARCHIVE: &str = "-";
const NO_DIRECTORY: &str = r#"[ -d "$1" ] || echo none"#;
const NOTHING: &str = r#"[ -e "$1" ] || echo none"#;
const NONE: &str = "none";
const TOP: &str = "/";

pub(super) struct Mount<'a> {
    pub(super) from: &'a Path,
    pub(super) to: &'a str,
    pub(super) read_only: bool,
}

pub(super) struct Shell<'a> {
    pub(super) command: &'a str,
    pub(super) environment: &'a [(&'a str, &'a str)],
}

pub(super) struct Sandbox<'a> {
    pub(super) host: Host<'a>,
    pub(super) image: &'a str,
    pub(super) mounts: &'a [Mount<'a>],
    pub(super) working_directory: &'a str,
}

pub(super) struct Run<'a> {
    pub(super) shell: &'a Shell<'a>,
    pub(super) user: &'a str,
    pub(super) variables: &'a [(String, String)],
}

pub(super) struct Bringing<'a> {
    pub(super) image: &'a str,
    pub(super) working_directory: Option<&'a str>,
    pub(super) cpus: Option<f64>,
    pub(super) memory_megabytes: Option<u64>,
    pub(super) variables: &'a [(String, String)],
    pub(super) stays: Duration,
}

pub(super) struct Up<'a> {
    docker: &'a dyn Docker,
    files: &'a dyn FileSystem,
    name: String,
    user: String,
    working_directory: String,
}

impl<'a> Bringing<'a> {
    pub(super) fn of_the_task(environment: &'a Environment) -> Self {
        let configuration = &environment.configuration;
        Self {
            image: &environment.image,
            working_directory: configuration.working_directory.as_deref(),
            cpus: configuration.cpus,
            memory_megabytes: configuration.memory_megabytes,
            variables: &configuration.variables,
            stays: configuration
                .harness_time_limit
                .unwrap_or(FOREVER)
                .min(FOREVER)
                .saturating_add(configuration.verifier_time_limit.min(FOREVER))
                .saturating_add(longest(configuration.health_check.as_ref()))
                .saturating_add(COPYING),
        }
    }

    pub(super) fn of_the_verifier(image: &'a str, configuration: &'a TaskConfiguration) -> Self {
        let own = configuration.verifier_environment.as_ref();
        Self {
            image,
            working_directory: own.and_then(|own| own.working_directory.as_deref()),
            cpus: own.and_then(|own| own.cpus),
            memory_megabytes: own.and_then(|own| own.memory_megabytes),
            variables: own.map_or(&[][..], |own| own.variables.as_slice()),
            stays: configuration
                .verifier_time_limit
                .min(FOREVER)
                .saturating_add(longest(own.and_then(|own| own.health_check.as_ref())))
                .saturating_add(COPYING),
        }
    }
}

fn longest(health_check: Option<&HealthCheck>) -> Duration {
    health_check
        .map_or(Duration::ZERO, HealthCheck::longest)
        .min(FOREVER)
}

impl Sandbox<'_> {
    pub(super) fn run(
        &self,
        shell: &Shell<'_>,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, SandboxError> {
        let name = named();
        let mut docker = self.docker_run(&name);
        shell.given_to(&mut docker);
        docker.arg(self.image);
        shell.timed(&mut docker, limits);
        let ended = self.host.docker.run(docker, "", running, limits);
        if matches!(ended, Ok(Ending::TimedOut) | Err(SandboxError::Stopped)) {
            remove(self.host.docker, &name);
        }
        started(ended, running)
    }

    pub(super) fn docker_run(&self, name: &str) -> Command {
        let mut docker = docker_run(self.host.outputs_directory, name);
        docker
            .arg("--user")
            .arg(format!("{}:{}", Uid::effective(), Gid::effective()))
            .args(["--workdir", self.working_directory]);
        for mount in self.mounts {
            docker.arg("--volume").arg(mount.written());
        }
        docker
    }
}

impl<'a> Up<'a> {
    pub(super) fn start(
        host: Host<'a>,
        bringing: &Bringing<'_>,
        running: &Path,
    ) -> Result<Self, SandboxError> {
        let name = named();
        let mut docker = docker_run(host.outputs_directory, &name);
        docker
            .args(
                bringing
                    .working_directory
                    .into_iter()
                    .flat_map(|directory| ["--workdir", directory]),
            )
            .args(
                bringing
                    .cpus
                    .iter()
                    .flat_map(|cpus| ["--cpus".to_owned(), cpus.to_string()]),
            )
            .args(
                bringing
                    .memory_megabytes
                    .iter()
                    .flat_map(|megabytes| ["--memory".to_owned(), format!("{megabytes}m")]),
            )
            .args(["--env-file", STANDARD_INPUT])
            .args(["--detach", bringing.image, "sleep"])
            .arg(bringing.stays.as_secs().to_string());
        let said = running.join("stderr");
        let mut up = Self {
            docker: host.docker,
            files: host.outputs_directory.files(),
            name,
            user: String::new(),
            working_directory: String::new(),
        };
        let detached = host
            .docker
            .detach(docker, &file_of(bringing.variables), &said)?;
        if !matches!(detached, Ending::Exited(0)) {
            return Err(SandboxError::Docker(said));
        }
        let mut inspect = Command::new("docker");
        inspect.args(["inspect", "--format", SETTINGS, &up.name]);
        host.docker
            .print(inspect)
            .map(move |printed| {
                let mut lines = printed.lines();
                let user = lines.next().unwrap_or_default();
                let working_directory = lines.next().unwrap_or_default().trim_matches('/');
                up.user = if user.is_empty() { ROOT } else { user }.to_owned();
                up.working_directory = format!("/{working_directory}");
                up
            })
            .map_err(SandboxError::Io)
    }

    pub(super) fn user(&self) -> &str {
        &self.user
    }

    pub(super) fn files(&self) -> &'a dyn FileSystem {
        self.files
    }

    pub(super) fn run(
        &self,
        run: &Run<'_>,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, SandboxError> {
        let mut docker = Command::new("docker");
        docker.args(["exec", "--env-file", STANDARD_INPUT]);
        run.shell.given_to(&mut docker);
        docker.args(["--user", run.user, &self.name]);
        run.shell.timed(&mut docker, limits);
        let ended = self
            .docker
            .run(docker, &file_of(run.variables), running, limits);
        started(ended, running)
    }

    pub(super) fn put(&self, from: &Path, to: &str, owner: &str) -> Result<(), SandboxError> {
        self.place(EMPTIED, &from.join("."), to, owner)
    }

    pub(super) fn put_file(&self, from: &Path, to: &str, owner: &str) -> Result<(), SandboxError> {
        self.place(ROOM_MADE, from, to, owner)
    }

    fn place(&self, room: &str, from: &Path, to: &str, owner: &str) -> Result<(), SandboxError> {
        let mut emptied = self.as_root();
        emptied.args(["sh", "-c", room, "sh", to]);
        let mut copied = Command::new("docker");
        copied
            .arg("cp")
            .arg(from)
            .arg(format!("{}:{to}", self.name));
        let mut owned = self.as_root();
        owned.args(["chown", "-R", owner, to]);
        [emptied, copied, owned]
            .into_iter()
            .try_for_each(|docker| self.docker.print(docker).map(drop))
            .map_err(SandboxError::Io)
    }

    pub(super) fn run_as_root(&self, script: &str, arguments: &[&str]) -> Result<(), SandboxError> {
        let mut docker = self.as_root();
        docker.args(["sh", "-c", script, "sh"]).args(arguments);
        self.docker
            .print(docker)
            .map(drop)
            .map_err(SandboxError::Io)
    }

    pub(super) fn end_everything(&self) {
        let mut docker = self.as_root();
        docker.args(["sh", "-c", END_EVERYTHING]);
        let _ = self.docker.print(docker);
    }

    pub(super) fn changes(&self) -> Vec<String> {
        let mut docker = Command::new("docker");
        docker.args(["diff", &self.name]);
        self.docker
            .print(docker)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    pub(super) fn keep_work(&self, work: &Path) -> Result<(), SandboxError> {
        let from = &self.working_directory;
        let kept = if from == TOP {
            Ok(())
        } else {
            self.take_out_of(from, work)
        };
        kept.map_err(|error| SandboxError::NotCopiedOut {
            left: Copied::Work(from.clone()),
            error,
        })
    }

    pub(super) fn copy_what_is_at(&self, from: &str, through: &Path, to: &Path) -> io::Result<()> {
        let _ = self.files.remove_tree(through);
        let copied = self
            .unpack(&[FOLLOW_LINK], from, through)
            .or_else(|error| self.unless_absent(NOTHING, from, error));
        let _ = self
            .files
            .list(through)
            .ok()
            .and_then(|names| names.into_iter().next())
            .map(|name| self.files.rename(&through.join(name), to));
        let _ = self.files.remove_tree(through);
        copied
    }

    pub(super) fn take_out_of(&self, from: &str, to: &Path) -> io::Result<()> {
        self.unpack(&[], &format!("{from}/."), to)
            .or_else(|error| self.unless_absent(NO_DIRECTORY, from, error))
    }

    fn unless_absent(&self, absent: &str, path: &str, error: io::Error) -> io::Result<()> {
        let mut docker = Command::new("docker");
        docker
            .args(["exec", "--workdir", TOP, "--user", ROOT, &self.name])
            .args(["sh", "-c", absent, "sh", path]);
        let said = self.docker.print(docker);
        if said.is_ok_and(|printed| printed.trim() == NONE) {
            Ok(())
        } else {
            Err(error)
        }
    }

    fn unpack(&self, flags: &[&str], from: &str, into: &Path) -> io::Result<()> {
        let mut docker = Command::new("docker");
        docker
            .arg("cp")
            .args(flags)
            .arg(format!("{}:{from}", self.name))
            .arg(ARCHIVE);
        let unpacked = self.docker.unpack(docker, into);
        make_removable(self.files, into);
        unpacked
    }

    fn as_root(&self) -> Command {
        let mut docker = Command::new("docker");
        docker.args(["exec", "--user", ROOT, &self.name]);
        docker
    }
}

impl Drop for Up<'_> {
    fn drop(&mut self) {
        remove(self.docker, &self.name);
    }
}

impl Shell<'_> {
    pub(super) fn given_to(&self, docker: &mut Command) {
        for &(variable, value) in self.environment {
            docker.args(["--env", variable]).env(variable, value);
        }
    }

    pub(super) fn timed(&self, docker: &mut Command, limits: Limits<'_>) {
        docker
            .args(["sh", "-c", TIMER, self.command])
            .arg(seconds_of_the_timer(limits.deadline).to_string());
    }
}

impl Mount<'_> {
    pub(super) fn written(&self) -> OsString {
        let mut written = self.from.as_os_str().to_owned();
        written.push(":");
        written.push(self.to);
        if self.read_only {
            written.push(":ro");
        }
        written
    }
}

fn docker_run(outputs_directory: &OutputsDirectory, name: &str) -> Command {
    let mut docker = Command::new("docker");
    docker
        .args(["run", "--rm", "--pull", "never", "--name", name, "--label"])
        .arg(left::label(outputs_directory))
        .args(["--add-host", MACHINE]);
    docker
}

pub(super) fn file_of(variables: &[(String, String)]) -> String {
    variables
        .iter()
        .fold(String::new(), |mut file, (name, value)| {
            let _ = writeln!(file, "{name}={value}");
            file
        })
}

fn named() -> String {
    format!("cladenc-{:016x}", rand::random::<u64>())
}

pub(super) fn started(
    ended: Result<Ending, SandboxError>,
    running: &Path,
) -> Result<Ending, SandboxError> {
    ended.and_then(|ending| match ending {
        Ending::Exited(NOT_STARTED) => Err(SandboxError::Docker(running.join("stderr"))),
        other => Ok(other),
    })
}

pub(super) fn seconds_of_the_timer(deadline: Instant) -> u64 {
    let left = deadline.saturating_duration_since(Instant::now());
    left.as_secs() + u64::from(left.subsec_nanos() > 0) + LEEWAY
}

fn remove(docker: &dyn Docker, name: &str) {
    let mut remove = Command::new("docker");
    remove.args(["rm", "--force", name]);
    let _ = docker.print(remove);
}
