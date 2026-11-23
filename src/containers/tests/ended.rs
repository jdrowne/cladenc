use std::path::{Path, PathBuf};

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
