use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use super::ContainerError;
use super::program::{self, Ending, deadline};
use crate::workspace::Workspace;

pub fn build_image(
    workspace: &Workspace,
    dockerfile: &Path,
    image: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let kept = workspace
        .start_image(dockerfile)
        .map_err(ContainerError::Io)?;
    let mut docker = Command::new("docker");
    docker
        .args(["build", "--tag", image, "--file"])
        .arg(kept.join("Dockerfile"))
        .arg(kept.join("context"));
    match program::run(docker, &kept, deadline(time_limit), stop)? {
        Ending::Exited(0) => Ok(()),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}
