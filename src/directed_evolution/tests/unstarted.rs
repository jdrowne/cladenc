use crate::containers::ContainerError;
use crate::directed_evolution::{self, DirectedEvolutionError};

#[test]
fn a_directed_evolution_asked_to_stop_before_its_first_record_is_said_not_to_have_started() {
    assert!(matches!(
        directed_evolution::not_started(DirectedEvolutionError::Asked),
        DirectedEvolutionError::NotStarted
    ));
}

#[test]
fn a_directed_evolution_stopped_while_its_image_is_built_is_said_not_to_have_started() {
    assert!(matches!(
        directed_evolution::not_started(DirectedEvolutionError::Image(ContainerError::Stopped)),
        DirectedEvolutionError::NotStarted
    ));
}

#[test]
fn a_directed_evolution_whose_image_is_not_built_keeps_the_error_of_its_image() {
    assert!(matches!(
        directed_evolution::not_started(DirectedEvolutionError::Image(ContainerError::TimeLimit)),
        DirectedEvolutionError::Image(ContainerError::TimeLimit)
    ));
}
