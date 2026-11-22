use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::ContainerError;
use super::left;
use super::program::{self, Ending, LEEWAY, Limits};
use crate::benchmark::ENVIRONMENT;
use crate::outputs_directory::OutputsDirectory;

const MACHINE: &str = "{{.NCPU}} {{.MemTotal}}";
const BYTES_IN_A_MEGABYTE: u64 = 1_048_576;

pub fn build_image(
    outputs_directory: &OutputsDirectory,
    dockerfile: &Path,
    image: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let kept = outputs_directory
        .start_image(dockerfile)
        .map_err(ContainerError::Io)?;
    let mut build = Command::new("timeout");
    build
        .args(["-s", "KILL"])
        .arg(time_limit.saturating_add(LEEWAY).to_string())
        .args(["docker", "build", "--tag", image, "--file"])
        .arg(kept.join("Dockerfile"))
        .arg(kept.join("context"));
    program::run(build, "", &kept, Limits::starting_now(time_limit, stop)).and_then(built)
}

pub fn build_environment(
    outputs_directory: &OutputsDirectory,
    task: &str,
    image: &str,
    time_limit: Duration,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let kept = outputs_directory
        .start_environment(task)
        .map_err(ContainerError::Io)?;
    let mut build = Command::new("docker");
    build
        .args(["build", "--tag", image, "--label"])
        .arg(format!("cladenc.task={task}"))
        .arg("--label")
        .arg(left::label(outputs_directory))
        .arg(outputs_directory.task(task).join(ENVIRONMENT));
    program::run(build, "", &kept, Limits::within(time_limit, stop)).and_then(built)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Machine {
    pub cpus: f64,
    pub memory_megabytes: u64,
}

pub fn machine() -> io::Result<Machine> {
    let mut info = Command::new("docker");
    info.args(["info", "--format", MACHINE]);
    left::printed_by(info).and_then(|printed| {
        let mut numbers = printed.split_whitespace();
        let cpus = numbers.next().and_then(|cpus| cpus.parse().ok());
        let memory = numbers.next().and_then(|bytes| bytes.parse::<u64>().ok());
        cpus.zip(memory)
            .map(|(cpus, bytes)| Machine {
                cpus,
                memory_megabytes: bytes / BYTES_IN_A_MEGABYTE,
            })
            .ok_or_else(|| {
                io::Error::other("Docker did not say how many CPUs and how much memory it has")
            })
    })
}

pub fn remove_images(images: &[String]) {
    let _ = Command::new("docker")
        .args(["image", "rm", "--force"])
        .args(images)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn find_image(image: &str) -> io::Result<()> {
    let mut inspect = Command::new("docker");
    inspect.args(["image", "inspect", "--format", "{{.Id}}", image]);
    left::printed_by(inspect).map(drop)
}

fn built(ending: Ending) -> Result<(), ContainerError> {
    match ending {
        Ending::Exited(0) => Ok(()),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}
