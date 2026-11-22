use std::fs::File;
use std::io::Write;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use super::ContainerError;

pub(super) const LOOK: Duration = Duration::from_millis(20);

pub(super) const FOREVER: Duration = Duration::from_secs(u32::MAX as u64);

pub(super) const LEEWAY: u64 = 5;

pub(super) fn seconds_since(started: Instant) -> f64 {
    (started.elapsed().as_secs_f64() * 1000.0).round() / 1000.0
}

pub(super) enum Ending {
    Exited(i32),
    TimedOut,
}

#[derive(Clone, Copy)]
pub(super) struct Limits<'a> {
    pub(super) deadline: Instant,
    stop: &'a AtomicBool,
}

impl<'a> Limits<'a> {
    pub(super) fn starting_now(time_limit: u64, stop: &'a AtomicBool) -> Self {
        Self::within(Duration::from_secs(time_limit), stop)
    }

    pub(super) fn within(time_limit: Duration, stop: &'a AtomicBool) -> Self {
        Self {
            deadline: Instant::now() + time_limit.min(FOREVER),
            stop,
        }
    }
}

pub(super) fn run(
    mut command: Command,
    input: &str,
    running: &Path,
    limits: Limits<'_>,
) -> Result<Ending, ContainerError> {
    let mut program = File::create(running.join("stdout"))
        .and_then(|stdout| File::create(running.join("stderr")).map(|stderr| (stdout, stderr)))
        .and_then(|(stdout, stderr)| {
            command
                .stdin(Stdio::piped())
                .stdout(stdout)
                .stderr(stderr)
                .process_group(0)
                .spawn()
        })
        .map_err(ContainerError::Io)?;
    give(&mut program, input);
    loop {
        if let Some(status) = program.try_wait().ok().flatten() {
            return status
                .code()
                .map(Ending::Exited)
                .ok_or_else(|| ContainerError::Signal(status.signal().unwrap_or_default()));
        }
        if limits.stop.load(Ordering::SeqCst) {
            kill(&mut program);
            return Err(ContainerError::Stopped);
        }
        if Instant::now() >= limits.deadline {
            kill(&mut program);
            return Ok(Ending::TimedOut);
        }
        thread::sleep(LOOK);
    }
}

pub(super) fn give(program: &mut Child, input: &str) {
    let _ = program
        .stdin
        .take()
        .map(|mut standard_input| standard_input.write_all(input.as_bytes()));
}

fn kill(program: &mut Child) {
    let group = Pid::from_raw(program.id().cast_signed());
    let _ = killpg(group, Signal::SIGKILL);
    let _ = program.wait();
}
