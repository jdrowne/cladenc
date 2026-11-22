use crate::cli;
use crate::containers::ContainerError;
use crate::directed_evolution::DirectedEvolutionError;
use crate::journal::{JournalError, Started};

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
    let error = DirectedEvolutionError::Build(ContainerError::Exit(1));
    assert_eq!(cli::stopped(error).0, 1);
}

#[test]
fn a_directed_evolution_one_of_whose_actions_fails_ends_with_status_one() {
    let error = DirectedEvolutionError::Action {
        action: Started::Attempt {
            harness: 3,
            task: "benchmark/t1".to_owned(),
        },
        error: ContainerError::TimeLimit,
    };
    assert_eq!(cli::stopped(error).0, 1);
}

#[test]
fn a_directed_evolution_that_did_not_start_ends_with_status_two() {
    assert_eq!(cli::stopped(DirectedEvolutionError::NotStarted).0, 2);
}

#[test]
fn a_directed_evolution_that_stops_says_why_in_the_words_of_its_error() {
    assert_eq!(
        cli::stopped(DirectedEvolutionError::Asked).1,
        "the directed evolution stopped before its end: it was asked to stop"
    );
}
