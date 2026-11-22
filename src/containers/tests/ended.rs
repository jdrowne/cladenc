use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::containers::ContainerError;
use crate::containers::container;
use crate::containers::program::Ending;

const RUNNING: &str = "/outputs/running/attempt-3-benchmark+t1";

fn said() -> PathBuf {
    Path::new(RUNNING).join("stderr")
}

fn started(ended: Result<Ending, ContainerError>) -> Result<Ending, ContainerError> {
    container::started(ended, Path::new(RUNNING))
}

fn sh(script: &str) -> Child {
    Command::new("sh")
        .args(["-c", script])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn a_container_that_docker_did_not_start_is_an_error_that_names_what_docker_said() {
    assert!(matches!(
        started(Ok(Ending::Exited(125))),
        Err(ContainerError::Docker(kept)) if kept == said()
    ));
}

#[test]
fn a_container_whose_command_exited_with_another_status_was_started() {
    assert!(matches!(
        started(Ok(Ending::Exited(1))),
        Ok(Ending::Exited(1))
    ));
}

#[test]
fn a_container_whose_command_ran_past_its_time_limit_was_started() {
    assert!(matches!(
        started(Ok(Ending::TimedOut)),
        Ok(Ending::TimedOut)
    ));
}

#[test]
fn a_container_that_was_stopped_is_not_said_to_be_one_that_docker_did_not_start() {
    assert!(matches!(
        started(Err(ContainerError::Stopped)),
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn a_command_that_leaves_its_container_running_and_exits_with_zero_has_brought_it_up() {
    assert!(matches!(
        container::detached(sh("exit 0"), "", said()),
        Ok(())
    ));
}

#[test]
fn a_command_that_leaves_no_container_running_is_an_error_that_names_what_docker_said() {
    assert!(matches!(
        container::detached(sh("exit 3"), "", said()),
        Err(ContainerError::Docker(kept)) if kept == said()
    ));
}

#[test]
fn a_command_that_a_signal_ends_before_its_container_runs_is_said_to_be_ended_by_that_signal() {
    assert!(matches!(
        container::detached(sh("kill -s KILL $$"), "", said()),
        Err(ContainerError::Signal(9))
    ));
}

#[test]
fn a_command_that_leaves_its_container_running_reads_what_it_is_given_on_its_standard_input() {
    let read = sh(r#"read line; test "$line" = "MODE=strict""#);
    assert!(matches!(
        container::detached(read, "MODE=strict\n", said()),
        Ok(())
    ));
}
