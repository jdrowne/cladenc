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
const TIMER: &str = r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) & wait $run"#;

pub(super) struct Mount<'a> {
    pub(super) from: &'a Path,
    pub(super) to: &'a str,
    pub(super) read_only: bool,
}

pub(super) struct Container<'a> {
    pub(super) outputs_directory: &'a OutputsDirectory,
    pub(super) image: &'a str,
    pub(super) command: &'a str,
    pub(super) mounts: &'a [Mount<'a>],
    pub(super) working_directory: &'a str,
    pub(super) environment: &'a [(&'a str, &'a str)],
}

impl Container<'_> {
    pub(super) fn run(&self, running: &Path, limits: Limits<'_>) -> Result<Ending, ContainerError> {
        let seconds = seconds_of_the_timer(limits.deadline);
        let name = format!("cladenc-{:016x}", rand::random::<u64>());
        let mut docker = Command::new("docker");
        docker
            .args(["run", "--rm", "--pull", "never", "--name", &name, "--label"])
            .arg(left::label(self.outputs_directory))
            .arg("--user")
            .arg(format!("{}:{}", Uid::effective(), Gid::effective()))
            .args(["--add-host", MACHINE])
            .args(["--workdir", self.working_directory]);
        for mount in self.mounts {
            docker.arg("--volume").arg(mount.written());
        }
        for &(variable, value) in self.environment {
            docker.args(["--env", variable]).env(variable, value);
        }
        docker
            .args([self.image, "sh", "-c", TIMER, self.command])
            .arg(seconds.to_string());
        let ended = program::run(docker, running, limits);
        if matches!(ended, Ok(Ending::TimedOut) | Err(ContainerError::Stopped)) {
            remove(&name);
        }
        ended.and_then(|ending| match ending {
            Ending::Exited(NOT_STARTED) => Err(ContainerError::Docker(running.join("stderr"))),
            started => Ok(started),
        })
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
