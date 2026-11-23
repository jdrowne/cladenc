use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::containers::program::{self, Ending, Limits};
use crate::containers::{ContainerError, Docker, Program};

fn asleep() -> Command {
    let mut asleep = Command::new("sh");
    asleep.args(["-c", "sleep 300"]);
    asleep
}

#[test]
fn a_program_that_a_signal_ends_is_said_to_be_ended_by_that_signal() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let mut killed = Command::new("sh");
    killed.args(["-c", "kill -s KILL $$"]);
    let ended = Program.run(
        killed,
        "",
        directory.path(),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(ended, Err(ContainerError::Signal(9))));
}

#[test]
fn a_program_still_running_once_a_stop_is_asked_for_is_ended_and_said_to_be_stopped() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(true);
    let ended = Program.run(
        asleep(),
        "",
        directory.path(),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(ended, Err(ContainerError::Stopped)));
}

#[test]
fn a_program_still_running_past_its_time_limit_is_ended_and_said_to_have_timed_out() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let ended = Program.run(
        asleep(),
        "",
        directory.path(),
        Limits::starting_now(0, &stop),
    );
    assert!(matches!(ended, Ok(Ending::TimedOut)));
}

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
