use std::path::{Path, PathBuf};
use std::process::Command;

use crate::containers::container;
use crate::containers::program::Ending;
use crate::containers::{ContainerError, Docker, Program};

const RUNNING: &str = "/outputs/running/attempt-3-benchmark+t1";

fn said() -> PathBuf {
    Path::new(RUNNING).join("stderr")
}

fn started(ended: Result<Ending, ContainerError>) -> Result<Ending, ContainerError> {
    container::started(ended, Path::new(RUNNING))
}

fn detached(script: &str, input: &str) -> Result<Ending, ContainerError> {
    let directory = test_inputs::temporary_directory();
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    Program.detach(command, input, &directory.path().join("stderr"))
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
fn a_command_that_leaves_its_container_running_is_said_to_have_exited_with_zero() {
    assert!(matches!(detached("exit 0", ""), Ok(Ending::Exited(0))));
}

#[test]
fn a_command_that_leaves_no_container_running_is_said_to_have_exited_with_its_status() {
    assert!(matches!(detached("exit 3", ""), Ok(Ending::Exited(3))));
}

#[test]
fn a_command_that_a_signal_ends_before_its_container_runs_is_said_to_be_ended_by_that_signal() {
    assert!(matches!(
        detached("kill -s KILL $$", ""),
        Err(ContainerError::Signal(9))
    ));
}

#[test]
fn a_command_that_leaves_its_container_running_reads_what_it_is_given_on_its_standard_input() {
    assert!(matches!(
        detached(
            r#"read line; test "$line" = "MODE=strict""#,
            "MODE=strict\n"
        ),
        Ok(Ending::Exited(0))
    ));
}

#[test]
fn a_command_with_no_file_to_keep_what_it_says_is_not_started() {
    let directory = test_inputs::temporary_directory();
    let said = directory.path().join("missing").join("stderr");
    assert!(matches!(
        Program.detach(Command::new("sh"), "", &said),
        Err(ContainerError::Io(_))
    ));
}
