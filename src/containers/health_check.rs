use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::ContainerError;
use super::container::{Run, Shell, Up};
use super::program::{Ending, FOREVER, LOOK, Limits};
use crate::benchmark::HealthCheck;

pub(super) fn passes(
    up: &Up,
    health_check: &HealthCheck,
    kept: &Path,
    stop: &AtomicBool,
) -> Result<bool, ContainerError> {
    let shell = Shell {
        command: &health_check.command,
        environment: &[],
    };
    let run = Run {
        shell: &shell,
        user: up.user(),
        variables: &[],
    };
    passes_with(health_check, stop, || {
        let limits = Limits::within(health_check.time_limit, stop);
        up.run(&run, kept, limits)
            .map(|ending| matches!(ending, Ending::Exited(0)))
    })
}

pub(super) fn passes_with(
    health_check: &HealthCheck,
    stop: &AtomicBool,
    mut run: impl FnMut() -> Result<bool, ContainerError>,
) -> Result<bool, ContainerError> {
    let started = Instant::now();
    let mut failures = 0;
    loop {
        if run()? {
            return Ok(true);
        }
        let wait = if started.elapsed() < health_check.start_period {
            health_check.start_interval
        } else {
            failures += 1;
            if failures >= health_check.retries {
                return Ok(false);
            }
            health_check.interval
        };
        wait_for(wait, stop)?;
    }
}

fn wait_for(time: Duration, stop: &AtomicBool) -> Result<(), ContainerError> {
    let until = Instant::now() + time.min(FOREVER);
    loop {
        if stop.load(Ordering::SeqCst) {
            return Err(ContainerError::Stopped);
        }
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Ok(());
        }
        thread::sleep(left.min(LOOK));
    }
}
