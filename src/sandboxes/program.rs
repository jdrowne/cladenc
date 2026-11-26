use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use super::SandboxError;
use super::archive;
use super::docker::Docker;

pub(super) const LOOK: Duration = Duration::from_millis(20);

pub(super) const FOREVER: Duration = Duration::from_secs(u32::MAX as u64);

pub(super) const LEEWAY: u64 = 5;

pub(super) fn seconds_since(started: Instant) -> f64 {
    (started.elapsed().as_secs_f64() * 1000.0).round() / 1000.0
}

pub enum Ending {
    Exited(i32),
    TimedOut,
}

#[derive(Clone, Copy)]
pub struct Limits<'a> {
    pub(super) deadline: Instant,
    stop: &'a AtomicBool,
}

impl<'a> Limits<'a> {
    pub fn starting_now(time_limit: u64, stop: &'a AtomicBool) -> Self {
        Self::within(Duration::from_secs(time_limit), stop)
    }

    pub(super) fn within(time_limit: Duration, stop: &'a AtomicBool) -> Self {
        Self {
            deadline: Instant::now() + time_limit.min(FOREVER),
            stop,
        }
    }

    pub(super) fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }
}

pub struct Program;

impl Docker for Program {
    fn run(
        &self,
        mut command: Command,
        input: &str,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, SandboxError> {
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
            .map_err(SandboxError::Io)?;
        let _ = program
            .stdin
            .take()
            .map(|mut standard_input| standard_input.write_all(input.as_bytes()));
        loop {
            if let Some(status) = program.try_wait().ok().flatten() {
                return status
                    .code()
                    .map(Ending::Exited)
                    .ok_or_else(|| SandboxError::Signal(status.signal().unwrap_or_default()));
            }
            if limits.stopped() {
                kill(&mut program);
                return Err(SandboxError::Stopped);
            }
            if Instant::now() >= limits.deadline {
                kill(&mut program);
                return Ok(Ending::TimedOut);
            }
            thread::sleep(LOOK);
        }
    }

    fn detach(
        &self,
        mut command: Command,
        input: &str,
        said: &Path,
    ) -> Result<Ending, SandboxError> {
        let mut program = File::create(said)
            .and_then(|stderr| {
                command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(stderr)
                    .spawn()
            })
            .map_err(SandboxError::Io)?;
        let _ = program
            .stdin
            .take()
            .map(|mut standard_input| standard_input.write_all(input.as_bytes()));
        program.wait().map_err(SandboxError::Io).and_then(|status| {
            status
                .code()
                .map(Ending::Exited)
                .ok_or_else(|| SandboxError::Signal(status.signal().unwrap_or_default()))
        })
    }

    fn print(&self, mut command: Command) -> io::Result<String> {
        let output = command.stdin(Stdio::null()).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(io::Error::other(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ))
        }
    }

    fn unpack(&self, mut command: Command, into: &Path) -> io::Result<()> {
        let mut program = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let said = program.stderr.take().map(|mut said| {
            thread::spawn(move || {
                let mut text = Vec::new();
                let _ = said.read_to_end(&mut text);
                String::from_utf8_lossy(&text).trim().to_owned()
            })
        });
        let unpacked = program.stdout.take().map_or(Ok(()), |mut printed| {
            let unpacked = archive::unpack(&mut printed, into);
            let _ = io::copy(&mut printed, &mut io::sink());
            unpacked
        });
        let said = said.and_then(|said| said.join().ok()).unwrap_or_default();
        program.wait().and_then(|status| {
            if status.success() {
                unpacked
            } else if said.is_empty() {
                Err(io::Error::other(status.to_string()))
            } else {
                Err(io::Error::other(said))
            }
        })
    }
}

fn kill(program: &mut Child) {
    let group = Pid::from_raw(program.id().cast_signed());
    let _ = killpg(group, Signal::SIGKILL);
    let _ = program.wait();
}
