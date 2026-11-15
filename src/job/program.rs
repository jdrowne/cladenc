use std::fs::File;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use super::JobError;

const LOOK: Duration = Duration::from_millis(20);

pub(super) enum Ending {
    Exited(i32),
    TimedOut,
}

pub(super) fn deadline(time_limit: u64) -> Instant {
    Instant::now() + Duration::from_secs(time_limit)
}

pub(super) fn run(
    mut command: Command,
    job: &Path,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<Ending, JobError> {
    let mut program = File::create(job.join("stdout"))
        .and_then(|stdout| File::create(job.join("stderr")).map(|stderr| (stdout, stderr)))
        .and_then(|(stdout, stderr)| {
            command
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(stderr)
                .process_group(0)
                .spawn()
        })
        .map_err(JobError::Io)?;
    loop {
        if let Some(status) = program.try_wait().ok().flatten() {
            return status
                .code()
                .map(Ending::Exited)
                .ok_or_else(|| JobError::Signal(status.signal().unwrap_or_default()));
        }
        if stop.load(Ordering::SeqCst) {
            kill(&mut program);
            return Err(JobError::Stopped);
        }
        if Instant::now() >= deadline {
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
