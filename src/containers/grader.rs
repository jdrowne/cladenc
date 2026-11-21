use std::path::Path;

use super::ContainerError;
use super::container::{Shell, Up};
use super::program::{Ending, Limits};
use super::reward;
use crate::benchmark::TESTS;
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const GIVEN_TESTS: &str = "/tests";
const LOGS: &str = "/logs";
const VERIFIER: &str = "/logs/verifier";
const GRADER: Shell<'static> = Shell {
    command: "/tests/test.sh",
    environment: &[],
};

pub(super) fn run(
    outputs_directory: &OutputsDirectory,
    up: &Up,
    task: &str,
    running: &Path,
    limits: Limits<'_>,
) -> Result<AttemptResult, ContainerError> {
    let grader = running.join("grader");
    let verifier = grader.join("logs/verifier");
    up.copy_into(&outputs_directory.task(task).join(TESTS), GIVEN_TESTS)
        .and_then(|()| up.copy_into(&grader.join("logs"), LOGS))
        .and_then(|()| up.run(&GRADER, &grader, limits))
        .map(|ending| {
            up.copy_out_of(VERIFIER, &verifier);
            match ending {
                Ending::TimedOut => AttemptResult::TimedOut,
                Ending::Exited(_) if reward::completed(&verifier) => AttemptResult::Completed,
                Ending::Exited(_) => AttemptResult::NotCompleted,
            }
        })
}
