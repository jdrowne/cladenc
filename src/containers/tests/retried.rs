use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::containers::health_check::{self, Backoff};
use crate::containers::{Checked, ContainerError, Retry};

const A_MOMENT: Duration = Duration::from_millis(40);
const A_MILLISECOND: Duration = Duration::from_millis(1);

struct Tried {
    passed: Result<Option<u64>, ContainerError>,
    tries: u64,
    told: Vec<Retry>,
    took: Duration,
}

fn tried_with(first_wait: Duration, max_retries: u64, failures: u64) -> Tried {
    let stop = AtomicBool::new(false);
    let backoff = Backoff {
        checked: Checked::Grader,
        first_wait,
        max_retries,
        stop: &stop,
    };
    let told = RefCell::new(Vec::new());
    let mut tries = 0;
    let started = Instant::now();
    let passed = health_check::retried(&backoff, &|retry| told.borrow_mut().push(retry), || {
        tries += 1;
        Ok((tries > failures).then_some(tries))
    });
    let took = started.elapsed();
    Tried {
        passed,
        tries,
        told: told.into_inner(),
        took,
    }
}

fn never_passing(first_wait: Duration, max_retries: u64) -> Tried {
    tried_with(first_wait, max_retries, u64::MAX)
}

#[test]
fn a_try_that_passes_the_first_time_is_not_made_again() {
    assert_eq!(tried_with(Duration::ZERO, 3, 0).tries, 1);
}

#[test]
fn a_try_that_passes_gives_what_it_made() {
    assert!(matches!(
        tried_with(Duration::ZERO, 3, 0).passed,
        Ok(Some(1))
    ));
}

#[test]
fn no_retry_is_told_where_the_first_try_passes() {
    assert_eq!(tried_with(Duration::ZERO, 3, 0).told, Vec::<Retry>::new());
}

#[test]
fn a_try_that_fails_is_made_again_until_it_passes() {
    assert_eq!(tried_with(Duration::ZERO, 3, 2).tries, 3);
}

#[test]
fn a_retry_that_passes_gives_what_it_made() {
    assert!(matches!(
        tried_with(Duration::ZERO, 3, 2).passed,
        Ok(Some(3))
    ));
}

#[test]
fn a_try_that_never_passes_is_made_once_and_then_as_many_times_as_the_most_retries() {
    assert_eq!(never_passing(Duration::ZERO, 3).tries, 4);
}

#[test]
fn a_try_that_never_passes_gives_nothing() {
    assert!(matches!(never_passing(Duration::ZERO, 3).passed, Ok(None)));
}

#[test]
fn a_try_that_passes_at_the_last_retry_has_passed() {
    assert!(matches!(
        tried_with(Duration::ZERO, 3, 3).passed,
        Ok(Some(4))
    ));
}

#[test]
fn with_0_retries_a_try_that_fails_is_not_made_again() {
    assert_eq!(never_passing(Duration::ZERO, 0).tries, 1);
}

#[test]
fn each_retry_is_told_with_its_number_the_most_retries_and_its_wait() {
    assert_eq!(
        never_passing(A_MILLISECOND, 2).told,
        [
            Retry {
                checked: Checked::Grader,
                number: 1,
                most: 2,
                wait: A_MILLISECOND,
            },
            Retry {
                checked: Checked::Grader,
                number: 2,
                most: 2,
                wait: A_MILLISECOND * 2,
            }
        ]
    );
}

#[test]
fn a_retry_is_told_with_the_health_check_that_did_not_pass() {
    let stop = AtomicBool::new(false);
    let backoff = Backoff {
        checked: Checked::Task,
        first_wait: Duration::ZERO,
        max_retries: 1,
        stop: &stop,
    };
    let told = RefCell::new(Vec::new());
    let _ = health_check::retried::<u64>(
        &backoff,
        &|retry: Retry| told.borrow_mut().push(retry.checked),
        || Ok(None),
    );
    assert_eq!(told.into_inner(), [Checked::Task]);
}

#[test]
fn the_wait_before_each_retry_is_twice_the_wait_before_the_last() {
    let waits: Vec<Duration> = never_passing(A_MILLISECOND, 4)
        .told
        .iter()
        .map(|retry| retry.wait)
        .collect();
    assert_eq!(
        waits,
        [
            A_MILLISECOND,
            A_MILLISECOND * 2,
            A_MILLISECOND * 4,
            A_MILLISECOND * 8
        ]
    );
}

#[test]
fn a_first_wait_of_0_is_followed_by_waits_of_0() {
    let waits: Vec<Duration> = never_passing(Duration::ZERO, 2)
        .told
        .iter()
        .map(|retry| retry.wait)
        .collect();
    assert_eq!(waits, [Duration::ZERO, Duration::ZERO]);
}

#[test]
fn the_waits_are_waited_before_the_retries_are_made() {
    assert!(tried_with(A_MOMENT, 2, 2).took >= A_MOMENT * 3);
}

#[test]
fn no_wait_follows_the_last_retry() {
    assert!(never_passing(Duration::from_secs(600), 0).took < Duration::from_secs(60));
}

#[test]
fn a_try_that_could_not_be_made_is_an_error() {
    let stop = AtomicBool::new(false);
    let backoff = Backoff {
        checked: Checked::Grader,
        first_wait: Duration::ZERO,
        max_retries: 3,
        stop: &stop,
    };
    let passed = health_check::retried::<u64>(&backoff, &|_| (), || Err(ContainerError::Exit(125)));
    assert!(matches!(passed, Err(ContainerError::Exit(125))));
}

#[test]
fn a_retry_that_waits_when_cladenc_is_asked_to_stop_is_stopped() {
    let stop = AtomicBool::new(true);
    let backoff = Backoff {
        checked: Checked::Grader,
        first_wait: Duration::from_secs(600),
        max_retries: 3,
        stop: &stop,
    };
    let passed = health_check::retried::<u64>(&backoff, &|_| (), || Ok(None));
    assert!(matches!(passed, Err(ContainerError::Stopped)));
}
