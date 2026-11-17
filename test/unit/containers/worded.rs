use std::error::Error;
use std::io;
use std::path::PathBuf;

use cladenc::containers::{self, ContainerError};

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn what_was_stopped_is_worded() {
    assert_eq!(worded(ContainerError::Stopped), "was stopped");
}

#[test]
fn what_could_not_be_carried_out_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(ContainerError::Io(io::Error::from_raw_os_error(2))),
        "could not be carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_container_that_docker_did_not_start_is_worded_with_the_file_that_holds_what_docker_said() {
    assert_eq!(
        worded(ContainerError::Docker(PathBuf::from(
            "/run/running/procreation-1/stderr"
        ))),
        "could not be carried out: Docker did not start its container, \
         and /run/running/procreation-1/stderr holds what Docker said"
    );
}

#[test]
fn the_instruction_says_where_the_source_code_to_improve_is() {
    assert!(
        containers::INSTRUCTION
            .contains("The current directory holds a copy of your own source code")
    );
}

#[test]
fn the_instruction_says_where_the_attempts_are() {
    assert!(containers::INSTRUCTION.contains(
        "than you do. /var/log/cladenc/attempts holds one directory for each benchmark, and in \
         it one directory for each task you have attempted. In the directory of a task, "
    ));
}

#[test]
fn the_instruction_says_what_each_attempt_keeps_of_the_task_and_of_the_harness() {
    assert!(containers::INSTRUCTION.contains(
        "the file named specification says what you were asked to do, the file named result \
         reads completed, not_completed or timed_out, the directory named work holds the \
         workspace of the task as you left it"
    ));
}

#[test]
fn the_instruction_says_where_what_the_grader_printed_is() {
    assert!(
        containers::INSTRUCTION
            .contains("the directory named grader holds what the grader of the task printed")
    );
}

#[test]
fn a_program_that_exited_with_another_status_is_worded() {
    assert_eq!(worded(ContainerError::Exit(2)), "exited with status 2");
}

#[test]
fn a_program_that_ran_past_its_time_limit_is_worded() {
    assert_eq!(worded(ContainerError::TimeLimit), "ran past its time limit");
}

#[test]
fn a_build_that_left_what_a_harness_cannot_hold_is_worded() {
    assert_eq!(
        worded(ContainerError::OtherFiles),
        "left something that is not a regular file, a directory or a symbolic link"
    );
}

#[test]
fn a_program_ended_by_a_signal_is_worded() {
    assert_eq!(worded(ContainerError::Signal(9)), "was ended by signal 9");
}
