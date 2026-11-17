use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use super::ContainerError;
use super::program::{self, Ending, LEEWAY, Limits};
use crate::run_directory::RunDirectory;

pub fn build_image(
    run_directory: &RunDirectory,
    dockerfile: &Path,
    image: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let kept = run_directory
        .start_image(dockerfile)
        .map_err(ContainerError::Io)?;
    let mut build = Command::new("timeout");
    build
        .args(["-s", "KILL"])
        .arg(time_limit.saturating_add(LEEWAY).to_string())
        .args(["docker", "build", "--tag", image, "--file"])
        .arg(kept.join("Dockerfile"))
        .arg(kept.join("context"));
    match program::run(build, &kept, Limits::starting_now(time_limit, stop))? {
        Ending::Exited(0) => Ok(()),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}
