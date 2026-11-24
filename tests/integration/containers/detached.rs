use std::fs;
use std::process::Command;

use cladenc::containers::{ContainerError, Docker, Ending, Program};
use tempfile::TempDir;

struct Detached {
    directory: TempDir,
    ended: Result<Ending, ContainerError>,
}

fn sh(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

fn detached(script: &str, input: &str) -> Detached {
    let directory = test_inputs::temporary_directory();
    let ended = Program.detach(sh(script), input, &directory.path().join("stderr"));
    Detached { directory, ended }
}

#[test]
fn a_command_that_leaves_its_container_running_is_said_to_have_exited_with_zero() {
    assert!(matches!(
        detached("exit 0", "").ended,
        Ok(Ending::Exited(0))
    ));
}

#[test]
fn a_command_that_leaves_no_container_running_is_said_to_have_exited_with_its_status() {
    assert!(matches!(
        detached("exit 3", "").ended,
        Ok(Ending::Exited(3))
    ));
}

#[test]
fn a_command_that_a_signal_ends_before_its_container_runs_is_said_to_be_ended_by_that_signal() {
    assert!(matches!(
        detached("kill -s KILL $$", "").ended,
        Err(ContainerError::Signal(9))
    ));
}

#[test]
fn a_command_that_sends_itself_a_hangup_before_its_container_runs_is_ended_by_it() {
    assert!(matches!(
        detached("kill -s HUP $$", "").ended,
        Err(ContainerError::Signal(1))
    ));
}

#[test]
fn a_command_that_sends_itself_a_quit_before_its_container_runs_is_ended_by_it() {
    assert!(matches!(
        detached("kill -s QUIT $$", "").ended,
        Err(ContainerError::Signal(3))
    ));
}

#[test]
fn a_command_that_leaves_its_container_running_reads_what_it_is_given_on_its_standard_input() {
    assert!(matches!(
        detached(
            r#"read line; test "$line" = "MODE=strict""#,
            "MODE=strict\n"
        )
        .ended,
        Ok(Ending::Exited(0))
    ));
}

#[test]
fn what_a_command_that_leaves_its_container_running_says_went_wrong_is_kept_in_the_file_named() {
    let detached = detached("echo printed; echo said >&2", "");
    assert_eq!(
        fs::read_to_string(detached.directory.path().join("stderr")).unwrap(),
        "said\n"
    );
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
