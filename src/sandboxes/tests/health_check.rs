use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::benchmark::HealthCheck;
use crate::sandboxes::SandboxError;
use crate::sandboxes::health_check;

const A_MOMENT: Duration = Duration::from_millis(40);

fn checked() -> HealthCheck {
    HealthCheck {
        command: "test -e /ready".to_owned(),
        interval: Duration::ZERO,
        time_limit: Duration::from_secs(30),
        start_period: Duration::ZERO,
        start_interval: Duration::ZERO,
        retries: 3,
    }
}

struct Checked {
    passed: Result<bool, SandboxError>,
    runs: u64,
    took: Duration,
}

fn checked_with(health_check: &HealthCheck, failures: u64) -> Checked {
    let mut runs = 0;
    let started = Instant::now();
    let passed = health_check::passes_with(health_check, &AtomicBool::new(false), || {
        runs += 1;
        Ok(runs > failures)
    });
    Checked {
        passed,
        runs,
        took: started.elapsed(),
    }
}

fn in_a_start_period() -> HealthCheck {
    HealthCheck {
        start_period: Duration::from_secs(60),
        retries: 1,
        ..checked()
    }
}

#[test]
fn a_health_check_that_passes_the_first_time_has_passed() {
    assert!(matches!(checked_with(&checked(), 0).passed, Ok(true)));
}

#[test]
fn a_health_check_that_passes_is_not_run_again() {
    assert_eq!(checked_with(&checked(), 0).runs, 1);
}

#[test]
fn a_health_check_that_fails_fewer_times_than_its_retries_and_then_passes_has_passed() {
    assert!(matches!(checked_with(&checked(), 2).passed, Ok(true)));
}

#[test]
fn a_health_check_that_fails_as_many_times_as_its_retries_has_not_passed() {
    assert!(matches!(checked_with(&checked(), 3).passed, Ok(false)));
}

#[test]
fn a_health_check_that_has_not_passed_is_run_as_many_times_as_its_retries() {
    assert_eq!(checked_with(&checked(), u64::MAX).runs, 3);
}

#[test]
fn a_health_check_with_1_retry_has_not_passed_once_it_fails() {
    let health_check = HealthCheck {
        retries: 1,
        ..checked()
    };
    assert_eq!(checked_with(&health_check, u64::MAX).runs, 1);
}

#[test]
fn a_failure_in_the_start_period_is_not_counted() {
    assert!(matches!(
        checked_with(&in_a_start_period(), 5).passed,
        Ok(true)
    ));
}

#[test]
fn a_health_check_that_fails_in_its_start_period_is_run_again_until_it_passes() {
    assert_eq!(checked_with(&in_a_start_period(), 5).runs, 6);
}

#[test]
fn a_failure_after_the_start_period_is_counted() {
    let health_check = HealthCheck {
        start_period: A_MOMENT,
        start_interval: A_MOMENT,
        retries: 1,
        ..checked()
    };
    assert!(matches!(
        checked_with(&health_check, u64::MAX).passed,
        Ok(false)
    ));
}

#[test]
fn a_health_check_waits_its_interval_before_it_is_run_again() {
    let health_check = HealthCheck {
        interval: A_MOMENT,
        ..checked()
    };
    assert!(checked_with(&health_check, 2).took >= A_MOMENT * 2);
}

#[test]
fn a_health_check_does_not_wait_once_it_has_not_passed() {
    let health_check = HealthCheck {
        interval: Duration::from_secs(600),
        retries: 1,
        ..checked()
    };
    assert!(checked_with(&health_check, u64::MAX).took < Duration::from_secs(60));
}

#[test]
fn a_health_check_waits_the_interval_of_its_start_period_within_it() {
    let health_check = HealthCheck {
        start_interval: A_MOMENT,
        interval: Duration::from_secs(600),
        ..in_a_start_period()
    };
    assert!(checked_with(&health_check, 2).took >= A_MOMENT * 2);
}

#[test]
fn a_health_check_does_not_wait_its_interval_within_its_start_period() {
    let health_check = HealthCheck {
        interval: Duration::from_secs(600),
        ..in_a_start_period()
    };
    assert!(checked_with(&health_check, 2).took < Duration::from_secs(60));
}

#[test]
fn a_health_check_that_could_not_be_run_is_an_error() {
    let passed = health_check::passes_with(&checked(), &AtomicBool::new(false), || {
        Err(SandboxError::Exit(125))
    });
    assert!(matches!(passed, Err(SandboxError::Exit(125))));
}

#[test]
fn a_health_check_that_waits_when_cladenc_is_asked_to_stop_is_stopped() {
    let health_check = HealthCheck {
        interval: Duration::from_secs(600),
        ..checked()
    };
    let passed = health_check::passes_with(&health_check, &AtomicBool::new(true), || Ok(false));
    assert!(matches!(passed, Err(SandboxError::Stopped)));
}
