use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use nix::unistd::{Gid, Uid};

use super::JobError;
use super::program::{self, Ending};

const NOT_STARTED: i32 = 125;
const MACHINE: &str = "host.docker.internal:host-gateway";

pub(super) struct Mount<'a> {
    pub from: &'a Path,
    pub to: &'a str,
    pub read_only: bool,
}

pub(super) struct Container<'a> {
    pub image: &'a str,
    pub command: &'a str,
    pub mounts: &'a [Mount<'a>],
    pub working_directory: &'a str,
    pub environment: &'a [(&'a str, &'a str)],
}

impl Container<'_> {
    pub(super) fn run(
        &self,
        job: &Path,
        deadline: Instant,
        stop: &AtomicBool,
    ) -> Result<Ending, JobError> {
        let name = format!("cladenc-{:016x}", rand::random::<u64>());
        let mut docker = Command::new("docker");
        docker
            .args(["run", "--rm", "--name", &name, "--user"])
            .arg(format!("{}:{}", Uid::effective(), Gid::effective()))
            .args(["--add-host", MACHINE])
            .args(["--workdir", self.working_directory]);
        for mount in self.mounts {
            docker.arg("--volume").arg(mount.written());
        }
        for &(variable, value) in self.environment {
            docker.args(["--env", variable]).env(variable, value);
        }
        docker.args([self.image, "sh", "-c", self.command]);
        let ended = program::run(docker, job, deadline, stop);
        if matches!(ended, Ok(Ending::TimedOut) | Err(JobError::Stopped)) {
            remove(&name);
        }
        ended.and_then(|ending| match ending {
            Ending::Exited(NOT_STARTED) => Err(JobError::Docker(job.join("stderr"))),
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

fn remove(name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "--force", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
