use std::error::Error;
use std::io;
use std::path::PathBuf;

use crate::benchmark::ConfigurationError;
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
fn a_task_whose_task_toml_cannot_be_obeyed_is_worded_with_its_task_and_why() {
    assert_eq!(
        worded(DirectedEvolutionError::Configuration {
            task: "benchmark/t1".to_owned(),
            error: ConfigurationError::NotSet {
                table: "environment",
                name: "API_TOKEN".to_owned(),
                variable: "TOKEN".to_owned(),
            },
        }),
        "task benchmark/t1 cannot be attempted: its task.toml gives API_TOKEN under \
         environment.env the value of TOKEN, which is not set where cladenc runs"
    );
}

#[test]
fn a_task_whose_copy_holds_no_task_toml_is_worded_with_its_task() {
    assert_eq!(
        worded(DirectedEvolutionError::Configuration {
            task: "benchmark/t1".to_owned(),
            error: ConfigurationError::Missing,
        }),
        "task benchmark/t1 cannot be attempted: its task.toml is not there"
    );
}

#[test]
fn an_image_that_is_not_on_the_machine_is_worded_with_its_task_and_what_docker_said() {
    assert_eq!(
        worded(DirectedEvolutionError::NotOnTheMachine {
            task: "benchmark/t1".to_owned(),
            image: "parser:3".to_owned(),
            error: io::Error::other("No such image: parser:3"),
        }),
        "the environment of task benchmark/t1 is not on the machine: its task.toml names the \
         image parser:3, and cladenc fetches no image: No such image: parser:3"
    );
}

#[test]
fn an_image_of_a_grader_that_is_not_on_the_machine_is_worded_with_its_task_and_what_docker_said() {
    assert_eq!(
        worded(DirectedEvolutionError::GraderNotOnTheMachine {
            task: "benchmark/t1".to_owned(),
            image: "grader:3".to_owned(),
            error: io::Error::other("No such image: grader:3"),
        }),
        "the environment of the grader of task benchmark/t1 is not on the machine: its task.toml \
         names the image grader:3, and cladenc fetches no image: No such image: grader:3"
    );
}

#[test]
fn an_environment_of_a_grader_that_was_not_built_is_worded_with_its_task_and_how_its_build_ended() {
    assert_eq!(
        worded(DirectedEvolutionError::GraderEnvironment {
            task: "benchmark/t1".to_owned(),
            error: ContainerError::Exit(1),
        }),
        "the environment of the grader of task benchmark/t1 was not built: its build exited with \
         status 1"
    );
}

#[test]
fn a_task_that_asks_for_more_than_the_machine_has_is_worded_with_what_it_asks_and_what_there_is() {
    assert_eq!(
        worded(DirectedEvolutionError::TooLarge {
            task: "benchmark/t1".to_owned(),
            key: "memory_mb",
            asked: "8192".to_owned(),
            has: "3916".to_owned(),
        }),
        "task benchmark/t1 asks for more than the machine has: 8192 under memory_mb in its \
         task.toml, where Docker has 3916"
    );
}

#[test]
fn a_machine_that_cannot_be_told_is_worded_with_why() {
    assert_eq!(
        worded(DirectedEvolutionError::Machine(io::Error::other(
            "Docker did not say"
        ))),
        "what the machine has cannot be told: Docker did not say"
    );
}

#[test]
fn a_directed_evolution_stopped_before_it_was_started_is_worded_with_what_resume_can_do() {
    assert_eq!(
        worded(DirectedEvolutionError::NotStarted),
        "the directed evolution was stopped before it was started, so `cladenc resume` has \
         nothing to continue"
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

#[test]
fn a_directed_evolution_that_a_grader_s_health_check_stopped_says_precisely_what_happened() {
    assert_eq!(
        worded(DirectedEvolutionError::Action {
            action: Started::Attempt {
                harness: 3,
                task: "parsing/t1".to_owned()
            },
            error: ContainerError::GraderHealthCheck {
                tries: 4,
                kept: PathBuf::from("/outputs/running/attempt-3-parsing+t1/grader/health-check"),
            }
        }),
        "the directed evolution stopped before its end: the attempt of harness 3 on task \
         parsing/t1 was not graded: the health check of the container of its grader did not \
         pass in 4 tries, so the grader was not run, and \
         /outputs/running/attempt-3-parsing+t1/grader/health-check holds what the health check \
         printed the last time it ran"
    );
}

#[test]
fn a_directed_evolution_that_a_task_s_health_check_stopped_says_precisely_what_happened() {
    assert_eq!(
        worded(DirectedEvolutionError::Action {
            action: Started::Attempt {
                harness: 3,
                task: "parsing/t1".to_owned()
            },
            error: ContainerError::TaskHealthCheck {
                tries: 4,
                kept: PathBuf::from("/outputs/running/attempt-3-parsing+t1/health-check"),
            }
        }),
        "the directed evolution stopped before its end: the attempt of harness 3 on task \
         parsing/t1 was not carried out: the health check of the task did not pass in 4 tries, \
         so the harness was not started, and /outputs/running/attempt-3-parsing+t1/health-check \
         holds what the health check printed the last time it ran"
    );
}
