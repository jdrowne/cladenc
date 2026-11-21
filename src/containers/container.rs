use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use nix::unistd::{Gid, Uid};

use super::ContainerError;
use super::left;
use super::program::{self, Ending, LEEWAY, Limits};
use crate::outputs_directory::OutputsDirectory;

const NOT_STARTED: i32 = 125;
const MACHINE: &str = "host.docker.internal:host-gateway";
const TIMER: &str =
    r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) > /dev/null 2>&1 & wait $run"#;

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

pub(super) struct Up {
    name: String,
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
        let ended = program::run(docker, running, limits);
        if matches!(ended, Ok(Ending::TimedOut) | Err(ContainerError::Stopped)) {
            remove(&name);
        }
        started(ended, running)
    }

    pub(super) fn start(&self, running: &Path, limits: Limits<'_>) -> Result<Up, ContainerError> {
        let up = Up { name: named() };
        let mut docker = self.docker_run(&up.name);
        docker
            .args(["--detach", self.image, "sleep"])
            .arg(seconds_of_the_timer(limits.deadline).to_string());
        match program::run(docker, running, limits)? {
            Ending::Exited(0) | Ending::TimedOut => Ok(up),
            Ending::Exited(_) => Err(ContainerError::Docker(running.join("stderr"))),
        }
    }

    fn docker_run(&self, name: &str) -> Command {
        let mut docker = Command::new("docker");
        docker
            .args(["run", "--rm", "--pull", "never", "--name", name, "--label"])
            .arg(left::label(self.outputs_directory))
            .arg("--user")
            .arg(format!("{}:{}", Uid::effective(), Gid::effective()))
            .args(["--add-host", MACHINE])
            .args(["--workdir", self.working_directory]);
        for mount in self.mounts {
            docker.arg("--volume").arg(mount.written());
        }
        docker
    }
}

impl Up {
    pub(super) fn run(
        &self,
        shell: &Shell<'_>,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let mut docker = Command::new("docker");
        docker.arg("exec");
        shell.given_to(&mut docker);
        docker.arg(&self.name);
        shell.timed(&mut docker, limits);
        started(program::run(docker, running, limits), running)
    }

    pub(super) fn copy_into(&self, from: &Path, to: &str) -> Result<(), ContainerError> {
        let mut docker = Command::new("docker");
        docker
            .args(["cp", "--archive"])
            .arg(from.join("."))
            .arg(format!("{}:{to}", self.name));
        left::printed_by(docker)
            .map(drop)
            .map_err(ContainerError::Io)
    }

    pub(super) fn copy_out_of(&self, from: &str, to: &Path) {
        let mut docker = Command::new("docker");
        docker
            .arg("cp")
            .arg(format!("{}:{from}/.", self.name))
            .arg(to);
        let _ = left::printed_by(docker);
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
