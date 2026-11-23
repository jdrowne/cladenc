use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::container::{Run, Shell, Up};
use super::program::{Ending, FOREVER, LOOK, Limits};
use super::{Checked, ContainerError, Retry, make_directory};
use crate::benchmark::HealthCheck;

pub(super) fn passes_where_given(
    up: &Up<'_>,
    health_check: Option<&HealthCheck>,
    kept: &Path,
    stop: &AtomicBool,
) -> Result<bool, ContainerError> {
    health_check.map_or(Ok(true), |health_check| {
        make_directory(kept);
        passes(up, health_check, kept, stop)
    })
}

fn passes(
    up: &Up<'_>,
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

pub(super) struct Backoff<'a> {
    pub(super) checked: Checked,
    pub(super) first_wait: Duration,
    pub(super) max_retries: u64,
    pub(super) stop: &'a AtomicBool,
}

pub(super) fn retried<T>(
    backoff: &Backoff<'_>,
    retrying: &dyn Fn(Retry),
    mut tried: impl FnMut() -> Result<Option<T>, ContainerError>,
) -> Result<Option<T>, ContainerError> {
    let mut wait = backoff.first_wait;
    let mut retries = 0;
    loop {
        let passed = tried()?;
        if passed.is_some() || retries >= backoff.max_retries {
            return Ok(passed);
        }
        retries += 1;
        retrying(Retry {
            checked: backoff.checked,
            number: retries,
            most: backoff.max_retries,
            wait,
        });
        wait_for(wait, backoff.stop)?;
        wait = wait.saturating_mul(2);
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
