use std::ffi::OsString;
use std::fmt::Write;
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use nix::unistd::{Gid, Uid};

use super::left;
use super::program::{self, Ending, FOREVER, LEEWAY, Limits};
use super::{ContainerError, Environment};
use crate::benchmark::HealthCheck;
use crate::outputs_directory::{OutputsDirectory, make_removable};

const NOT_STARTED: i32 = 125;
const MACHINE: &str = "host.docker.internal:host-gateway";
const TIMER: &str =
    r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) > /dev/null 2>&1 & wait $run"#;
const SETTINGS: &str = "{{println .Config.User}}{{.Config.WorkingDir}}";
const ROOT: &str = "0";
const STANDARD_INPUT: &str = "/dev/stdin";
const EMPTIED: &str = r#"rm -rf "$1" && mkdir -p "$1""#;
const END_EVERYTHING: &str = "kill -9 -1";
const COPYING: Duration = Duration::from_secs(600);

pub(super) struct Mount<'a> {
    pub(super) from: &'a Path,
    pub(super) to: &'a str,
    pub(super) read_only: bool,
}

pub(super) struct Shell<'a> {
    pub(super) command: &'a str,
    pub(super) environment: &'a [(&'a str, &'a str)],
}

pub(super) struct Container<'a> {
    pub(super) outputs_directory: &'a OutputsDirectory,
    pub(super) image: &'a str,
    pub(super) mounts: &'a [Mount<'a>],
    pub(super) working_directory: &'a str,
}

pub(super) struct Run<'a> {
    pub(super) shell: &'a Shell<'a>,
    pub(super) user: &'a str,
    pub(super) variables: &'a [(String, String)],
}

pub(super) struct Up {
    name: String,
    user: String,
    working_directory: String,
}

impl Container<'_> {
    pub(super) fn run(
        &self,
        shell: &Shell<'_>,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let name = named();
        let mut docker = self.docker_run(&name);
        shell.given_to(&mut docker);
        docker.arg(self.image);
        shell.timed(&mut docker, limits);
        let ended = program::run(docker, "", running, limits);
        if matches!(ended, Ok(Ending::TimedOut) | Err(ContainerError::Stopped)) {
            remove(&name);
        }
        started(ended, running)
    }

    fn docker_run(&self, name: &str) -> Command {
        let mut docker = docker_run(self.outputs_directory, name);
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

impl Up {
    pub(super) fn start(
        outputs_directory: &OutputsDirectory,
        environment: &Environment,
        running: &Path,
    ) -> Result<Self, ContainerError> {
        let configuration = &environment.configuration;
        let stays = configuration
            .harness_time_limit
            .unwrap_or(FOREVER)
            .min(FOREVER)
            .saturating_add(configuration.grader_time_limit.min(FOREVER))
            .saturating_add(
                configuration
                    .health_check
                    .as_ref()
                    .map_or(Duration::ZERO, HealthCheck::longest)
                    .min(FOREVER),
            )
            .saturating_add(COPYING);
        let mut up = Self {
            name: named(),
            user: String::new(),
            working_directory: String::new(),
        };
        let mut docker = docker_run(outputs_directory, &up.name);
        docker
            .args(
                configuration
                    .working_directory
                    .iter()
                    .flat_map(|directory| ["--workdir", directory]),
            )
            .args(
                configuration
                    .cpus
                    .iter()
                    .flat_map(|cpus| ["--cpus".to_owned(), cpus.to_string()]),
            )
            .args(
                configuration
                    .memory_megabytes
                    .iter()
                    .flat_map(|megabytes| ["--memory".to_owned(), format!("{megabytes}m")]),
            )
            .args(["--env-file", STANDARD_INPUT])
            .args(["--detach", &environment.image, "sleep"])
            .arg(stays.as_secs().to_string());
        detached(docker, &file_of(&configuration.variables), running)?;
        let mut inspect = Command::new("docker");
        inspect.args(["inspect", "--format", SETTINGS, &up.name]);
        left::printed_by(inspect)
            .map(move |printed| {
                let mut lines = printed.lines();
                let user = lines.next().unwrap_or_default();
                let working_directory = lines.next().unwrap_or_default().trim_matches('/');
                up.user = if user.is_empty() { ROOT } else { user }.to_owned();
                up.working_directory = format!("/{working_directory}");
                up
            })
            .map_err(ContainerError::Io)
    }

    pub(super) fn user(&self) -> &str {
        &self.user
    }

    pub(super) fn run(
        &self,
        run: &Run<'_>,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let mut docker = Command::new("docker");
        docker.args(["exec", "--env-file", STANDARD_INPUT]);
        run.shell.given_to(&mut docker);
        docker.args(["--user", run.user, &self.name]);
        run.shell.timed(&mut docker, limits);
        let ended = program::run(docker, &file_of(run.variables), running, limits);
        started(ended, running)
    }

    pub(super) fn put(&self, from: &Path, to: &str, owner: &str) -> Result<(), ContainerError> {
        let mut emptied = self.as_root();
        emptied.args(["sh", "-c", EMPTIED, "sh", to]);
        let mut copied = Command::new("docker");
        copied
            .arg("cp")
            .arg(from.join("."))
            .arg(format!("{}:{to}", self.name));
        let mut owned = self.as_root();
        owned.args(["chown", "-R", owner, to]);
        [emptied, copied, owned]
            .into_iter()
            .try_for_each(|docker| left::printed_by(docker).map(drop))
            .map_err(ContainerError::Io)
    }

    pub(super) fn end_everything(&self) {
        let mut docker = self.as_root();
        docker.args(["sh", "-c", END_EVERYTHING]);
        let _ = left::printed_by(docker);
    }

    pub(super) fn changes(&self) -> Vec<String> {
        let mut docker = Command::new("docker");
        docker.args(["diff", &self.name]);
        left::printed_by(docker)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    pub(super) fn keep_work(&self, work: &Path) {
        if self.working_directory != "/" {
            self.copy_out_of(&self.working_directory, work);
        }
    }

    pub(super) fn copy_out_of(&self, from: &str, to: &Path) {
        let mut docker = Command::new("docker");
        docker
            .arg("cp")
            .arg(format!("{}:{from}/.", self.name))
            .arg(to);
        let _ = left::printed_by(docker);
        make_removable(to);
    }

    fn as_root(&self) -> Command {
        let mut docker = Command::new("docker");
        docker.args(["exec", "--user", ROOT, &self.name]);
        docker
    }
}

impl Drop for Up {
    fn drop(&mut self) {
        remove(&self.name);
    }
}

impl Shell<'_> {
    fn given_to(&self, docker: &mut Command) {
        for &(variable, value) in self.environment {
            docker.args(["--env", variable]).env(variable, value);
        }
    }

    fn timed(&self, docker: &mut Command, limits: Limits<'_>) {
        docker
            .args(["sh", "-c", TIMER, self.command])
            .arg(seconds_of_the_timer(limits.deadline).to_string());
    }
}

impl Mount<'_> {
    fn written(&self) -> OsString {
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

fn detached(mut docker: Command, input: &str, running: &Path) -> Result<(), ContainerError> {
    let said = running.join("stderr");
    let status = File::create(&said)
        .and_then(|stderr| {
            docker
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(stderr)
                .spawn()
        })
        .and_then(|mut docker| {
            program::give(&mut docker, input);
            docker.wait()
        })
        .map_err(ContainerError::Io)?;
    match status.code() {
        Some(0) => Ok(()),
        Some(_) => Err(ContainerError::Docker(said)),
        None => Err(ContainerError::Signal(status.signal().unwrap_or_default())),
    }
}

fn named() -> String {
    format!("cladenc-{:016x}", rand::random::<u64>())
}

fn started(
    ended: Result<Ending, ContainerError>,
    running: &Path,
) -> Result<Ending, ContainerError> {
    ended.and_then(|ending| match ending {
        Ending::Exited(NOT_STARTED) => Err(ContainerError::Docker(running.join("stderr"))),
        other => Ok(other),
    })
}

fn seconds_of_the_timer(deadline: Instant) -> u64 {
    let left = deadline.saturating_duration_since(Instant::now());
    left.as_secs() + u64::from(left.subsec_nanos() > 0) + LEEWAY
}

fn remove(name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "--force", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
