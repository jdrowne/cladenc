use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::ContainerError;
use super::docker::{Docker, Host};
use super::left;
use super::program::{Ending, LEEWAY, Limits};
use crate::benchmark::{ENVIRONMENT, TESTS};

const MACHINE: &str = "{{.NCPU}} {{.MemTotal}}";
const BYTES_IN_A_MEGABYTE: u64 = 1_048_576;

pub fn build_image(
    host: Host<'_>,
    dockerfile: &Path,
    image: &str,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let kept = host
        .outputs_directory
        .start_image(dockerfile)
        .map_err(ContainerError::Io)?;
    let mut build = Command::new("timeout");
    build
        .args(["-s", "KILL"])
        .arg(time_limit.saturating_add(LEEWAY).to_string())
        .args(["docker", "build", "--tag", image, "--file"])
        .arg(kept.join("Dockerfile"))
        .arg(kept.join("context"));
    host.docker
        .run(build, "", &kept, Limits::starting_now(time_limit, stop))
        .and_then(built)
}

pub fn build_environment(
    host: Host<'_>,
    task: &str,
    image: &str,
    time_limit: Duration,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let context = host.outputs_directory.task(task).join(ENVIRONMENT);
    host.outputs_directory
        .start_environment(task)
        .map_err(ContainerError::Io)
        .and_then(|kept| {
            let limits = Limits::within(time_limit, stop);
            build(host, [task, image], &context, &kept, limits)
        })
}

pub fn build_verifier_environment(
    host: Host<'_>,
    task: &str,
    image: &str,
    time_limit: Duration,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let context = host.outputs_directory.task(task).join(TESTS);
    host.outputs_directory
        .start_verifier_environment(task)
        .map_err(ContainerError::Io)
        .and_then(|kept| {
            let limits = Limits::within(time_limit, stop);
            build(host, [task, image], &context, &kept, limits)
        })
}

fn build(
    host: Host<'_>,
    [task, image]: [&str; 2],
    context: &Path,
    kept: &Path,
    limits: Limits<'_>,
) -> Result<(), ContainerError> {
    let mut build = Command::new("docker");
    build
        .args(["build", "--tag", image, "--label"])
        .arg(format!("cladenc.task={task}"))
        .arg("--label")
        .arg(left::label(host.outputs_directory))
        .arg(context);
    host.docker.run(build, "", kept, limits).and_then(built)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Machine {
    pub cpus: f64,
    pub memory_megabytes: u64,
}

pub fn machine(docker: &dyn Docker) -> io::Result<Machine> {
    let mut info = Command::new("docker");
    info.args(["info", "--format", MACHINE]);
    docker.print(info).and_then(|printed| {
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

pub fn remove_images(docker: &dyn Docker, images: &[String]) {
    let mut remove = Command::new("docker");
    remove.args(["image", "rm", "--force"]).args(images);
    let _ = docker.print(remove);
}

pub fn find_image(docker: &dyn Docker, image: &str) -> io::Result<()> {
    let mut inspect = Command::new("docker");
    inspect.args(["image", "inspect", "--format", "{{.Id}}", image]);
    docker.print(inspect).map(drop)
}

pub(super) fn built(ending: Ending) -> Result<(), ContainerError> {
    match ending {
        Ending::Exited(0) => Ok(()),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}
