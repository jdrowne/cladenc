use std::fs::File;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use super::ContainerError;

const LOOK: Duration = Duration::from_millis(20);

pub(super) const LEEWAY: u64 = 5;

pub(super) enum Ending {
    Exited(i32),
    TimedOut,
}

#[derive(Clone, Copy)]
pub(super) struct Limits<'a> {
    pub deadline: Instant,
    pub stop: &'a AtomicBool,
}

impl<'a> Limits<'a> {
    pub(super) fn starting_now(time_limit: u64, stop: &'a AtomicBool) -> Self {
        Self {
            deadline: Instant::now() + Duration::from_secs(time_limit),
            stop,
        }
    }
}

pub(super) fn run(
    mut command: Command,
    running: &Path,
    limits: Limits<'_>,
) -> Result<Ending, ContainerError> {
    let mut program = File::create(running.join("stdout"))
        .and_then(|stdout| File::create(running.join("stderr")).map(|stderr| (stdout, stderr)))
        .and_then(|(stdout, stderr)| {
            command
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(stderr)
                .process_group(0)
                .spawn()
        })
        .map_err(ContainerError::Io)?;
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

fn kill(program: &mut Child) {
    let group = Pid::from_raw(program.id().cast_signed());
    let _ = killpg(group, Signal::SIGKILL);
    let _ = program.wait();
}
