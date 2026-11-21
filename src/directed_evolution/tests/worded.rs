use std::error::Error;
use std::io;
use std::path::PathBuf;

use crate::containers::ContainerError;
use crate::directed_evolution::DirectedEvolutionError;
use crate::journal::{JournalError, Started};

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn an_outputs_directory_that_cannot_be_made_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(DirectedEvolutionError::Io(io::Error::from_raw_os_error(17))),
        "the outputs directory cannot be made: File exists (os error 17)"
    );
}

#[test]
fn an_outputs_directory_that_cannot_be_used_is_worded_with_the_journal_s_reason() {
    assert_eq!(
        worded(DirectedEvolutionError::Unusable(JournalError::Locked)),
        "the outputs directory cannot be used: another cladenc is using it"
    );
}

#[test]
fn a_key_that_cannot_be_read_is_worded_with_its_file_and_what_the_file_has_to_hold() {
    assert_eq!(
        worded(DirectedEvolutionError::Key(PathBuf::from("/keys/model"))),
        "the key of the model cannot be read: /keys/model is not a file that holds a key on one \
         line and nothing else"
    );
}

#[test]
fn containers_that_cannot_be_removed_are_worded_with_what_docker_said() {
    assert_eq!(
        worded(DirectedEvolutionError::Left(io::Error::other(
            "Cannot connect to the Docker daemon"
        ))),
        "the containers that the directed evolution left running cannot be removed: Cannot \
         connect to the Docker daemon"
    );
}

#[test]
fn a_journal_that_could_not_be_written_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(DirectedEvolutionError::Journal(JournalError::Io(
            io::Error::from_raw_os_error(28)
        ))),
        "the directed evolution stopped before its end: its journal could not be written: \
         No space left on device (os error 28)"
    );
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_is_worded() {
    assert_eq!(
        worded(DirectedEvolutionError::Asked),
        "the directed evolution stopped before its end: it was asked to stop"
    );
}

#[test]
fn an_image_that_was_not_built_is_worded_with_how_its_build_ended() {
    assert_eq!(
        worded(DirectedEvolutionError::Image(ContainerError::Exit(1))),
        "the image of the directed evolution was not built: its build exited with status 1"
    );
}

#[test]
fn an_image_whose_build_took_too_long_is_worded_with_its_time_limit() {
    assert_eq!(
        worded(DirectedEvolutionError::Image(ContainerError::TimeLimit)),
        "the image of the directed evolution was not built: its build ran past its time limit"
    );
}

#[test]
fn an_image_whose_build_was_stopped_is_worded_as_stopped() {
    assert_eq!(
        worded(DirectedEvolutionError::Image(ContainerError::Stopped)),
        "the image of the directed evolution was not built: its build was stopped"
    );
}

#[test]
fn an_environment_that_was_not_built_is_worded_with_its_task_and_how_its_build_ended() {
    assert_eq!(
        worded(DirectedEvolutionError::Environment {
            task: "benchmark/t1".to_owned(),
            error: ContainerError::Exit(1),
        }),
        "the environment of task benchmark/t1 was not built: its build exited with status 1"
    );
}

#[test]
fn an_environment_whose_build_was_stopped_is_worded_as_stopped() {
    assert_eq!(
        worded(DirectedEvolutionError::Environment {
            task: "benchmark/t1".to_owned(),
            error: ContainerError::Stopped,
        }),
        "the environment of task benchmark/t1 was not built: its build was stopped"
    );
}

#[test]
fn a_directed_evolution_the_build_of_its_primogenitor_stopped_says_how_the_build_ended() {
    assert_eq!(
        worded(DirectedEvolutionError::Build(ContainerError::Exit(2))),
        "the directed evolution stopped before its end: the build of the primogenitor exited with \
         status 2"
    );
}

#[test]
fn a_directed_evolution_whose_primogenitor_took_too_long_to_build_says_so() {
    assert_eq!(
        worded(DirectedEvolutionError::Build(ContainerError::TimeLimit)),
        "the directed evolution stopped before its end: \
         the build of the primogenitor ran past its time limit"
    );
}

#[test]
fn a_directed_evolution_an_attempt_stopped_names_the_harness_and_the_task() {
    assert_eq!(
        worded(DirectedEvolutionError::Action {
            action: Started::Attempt {
                harness: 0,
                task: "t1".to_owned()
            },
            error: ContainerError::Exit(2)
        }),
        "the directed evolution stopped before its end: \
         the attempt of harness 0 on task t1 exited with status 2"
    );
}

#[test]
fn a_directed_evolution_a_procreation_stopped_names_the_progenitor_and_the_child() {
    assert_eq!(
        worded(DirectedEvolutionError::Action {
            action: Started::Procreation {
                child: 3,
                progenitor: 0
            },
            error: ContainerError::Signal(9)
        }),
        "the directed evolution stopped before its end: \
         the procreation of harness 3 by harness 0 was ended by signal 9"
    );
}

#[test]
fn a_directed_evolution_a_baseline_stopped_names_the_singleton_control_group_harness() {
    assert_eq!(
        worded(DirectedEvolutionError::Action {
            action: Started::Baseline {
                harness: 4,
                child: 3,
                progenitor: 1
            },
            error: ContainerError::Exit(2)
        }),
        "the directed evolution stopped before its end: the creation of harness 4, the \
         singleton control group harness, by the primogenitor from the source code of harness 1 \
         exited with status 2"
    );
}
