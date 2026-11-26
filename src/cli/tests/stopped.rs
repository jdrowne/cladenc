use std::io;
use std::path::PathBuf;

use crate::cli;
use crate::dataset::ConfigurationError;
use crate::directed_evolution::DirectedEvolutionError;
use crate::journal::{JournalError, Started};
use crate::sandboxes::SandboxError;

const TASK: &str = "dataset/t2";
const IMAGE: &str = "parser:3";

fn no_such_image() -> io::Error {
    io::Error::other("No such image: parser:3")
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_ends_with_status_one() {
    assert_eq!(cli::stopped(DirectedEvolutionError::Asked).0, 1);
}

#[test]
fn a_directed_evolution_whose_journal_cannot_be_written_ends_with_status_one() {
    let error = DirectedEvolutionError::Journal(JournalError::Locked);
    assert_eq!(cli::stopped(error).0, 1);
}

#[test]
fn a_directed_evolution_whose_primogenitor_is_not_built_ends_with_status_one() {
    let error = DirectedEvolutionError::Build(SandboxError::Exit(1));
    assert_eq!(cli::stopped(error).0, 1);
}

#[test]
fn a_directed_evolution_one_of_whose_actions_fails_ends_with_status_one() {
    let error = DirectedEvolutionError::Action {
        action: Started::Trial {
            harness: 3,
            task: "dataset/t1".to_owned(),
        },
        error: SandboxError::TimeLimit,
    };
    assert_eq!(cli::stopped(error).0, 1);
}

#[test]
fn a_directed_evolution_that_did_not_start_ends_with_status_two() {
    assert_eq!(cli::stopped(DirectedEvolutionError::NotStarted).0, 2);
}

#[test]
fn a_directed_evolution_whose_key_cannot_be_read_ends_with_status_two() {
    let error = DirectedEvolutionError::Key(PathBuf::from("/model-key"));
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_whose_image_is_not_built_ends_with_status_two() {
    let error = DirectedEvolutionError::Image(SandboxError::Exit(1));
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_a_task_whose_task_toml_cannot_be_obeyed_ends_with_status_two() {
    let error = DirectedEvolutionError::Configuration {
        task: TASK.to_owned(),
        error: ConfigurationError::Missing,
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_a_task_whose_image_is_not_on_the_machine_ends_with_status_two() {
    let error = DirectedEvolutionError::NotOnTheMachine {
        task: TASK.to_owned(),
        image: IMAGE.to_owned(),
        error: no_such_image(),
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_a_task_that_asks_for_more_than_the_machine_has_ends_with_status_two() {
    let error = DirectedEvolutionError::TooLarge {
        task: TASK.to_owned(),
        key: "cpus",
        asked: "100000".to_owned(),
        has: "4".to_owned(),
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_an_environment_that_is_not_built_ends_with_status_two() {
    let error = DirectedEvolutionError::Environment {
        task: TASK.to_owned(),
        error: SandboxError::Exit(1),
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_a_verifier_whose_image_is_not_on_the_machine_ends_with_status_two() {
    let error = DirectedEvolutionError::VerifierNotOnTheMachine {
        task: TASK.to_owned(),
        image: IMAGE.to_owned(),
        error: no_such_image(),
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_with_a_verifier_whose_environment_is_not_built_ends_with_status_two() {
    let error = DirectedEvolutionError::VerifierEnvironment {
        task: TASK.to_owned(),
        error: SandboxError::Exit(1),
    };
    assert_eq!(cli::stopped(error).0, 2);
}

#[test]
fn a_directed_evolution_that_stops_says_why_in_the_words_of_its_error() {
    assert_eq!(
        cli::stopped(DirectedEvolutionError::Asked).1,
        "the directed evolution stopped before its end: it was asked to stop"
    );
}
