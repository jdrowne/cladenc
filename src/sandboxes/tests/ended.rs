use std::path::{Path, PathBuf};

use crate::sandboxes::SandboxError;
use crate::sandboxes::program::Ending;
use crate::sandboxes::sandbox;

const RUNNING: &str = "/outputs/running/trial-3-dataset+t1";

fn said() -> PathBuf {
    Path::new(RUNNING).join("stderr")
}

fn started(ended: Result<Ending, SandboxError>) -> Result<Ending, SandboxError> {
    sandbox::started(ended, Path::new(RUNNING))
}

#[test]
fn a_sandbox_that_docker_did_not_start_is_an_error_that_names_what_docker_said() {
    assert!(matches!(
        started(Ok(Ending::Exited(125))),
        Err(SandboxError::Docker(kept)) if kept == said()
    ));
}

#[test]
fn a_sandbox_whose_command_exited_with_another_status_was_started() {
    assert!(matches!(
        started(Ok(Ending::Exited(1))),
        Ok(Ending::Exited(1))
    ));
}

#[test]
fn a_sandbox_whose_command_ran_past_its_time_limit_was_started() {
    assert!(matches!(
        started(Ok(Ending::TimedOut)),
        Ok(Ending::TimedOut)
    ));
}

#[test]
fn a_sandbox_that_was_stopped_is_not_said_to_be_one_that_docker_did_not_start() {
    assert!(matches!(
        started(Err(SandboxError::Stopped)),
        Err(SandboxError::Stopped)
    ));
}
