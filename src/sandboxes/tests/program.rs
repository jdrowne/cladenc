use std::time::{Duration, Instant};

use crate::sandboxes::program;

#[test]
fn the_seconds_since_a_moment_are_those_that_have_passed_since_it() {
    let started = Instant::now() - Duration::from_secs(2);
    assert!((2.0..3.0).contains(&program::seconds_since(started)));
}

#[test]
fn the_seconds_since_a_moment_are_rounded_to_the_millisecond() {
    let started = Instant::now() - Duration::from_millis(1234);
    let seconds = program::seconds_since(started).to_string();
    assert!(
        seconds
            .split('.')
            .nth(1)
            .is_none_or(|decimals| decimals.len() <= 3)
    );
}
