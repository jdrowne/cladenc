use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::container::{Run, Shell, Up};
use super::program::{Ending, Limits};
use super::reward;
use super::{Attempt, ContainerError};
use crate::benchmark::TESTS;
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const GIVEN_TESTS: &str = "/tests";
const VERIFIER: &str = "/logs/verifier";
const GRADER: Shell<'static> = Shell {
    command: "/tests/test.sh",
    environment: &[],
};

pub(super) fn run(
    outputs_directory: &OutputsDirectory,
    up: &Up,
    attempt: &Attempt<'_>,
    running: &Path,
    stop: &AtomicBool,
) -> Result<AttemptResult, ContainerError> {
    let configuration = &attempt.environment.configuration;
    let limits = Limits::within(configuration.grader_time_limit, stop);
    let grader = running.join("grader");
    let verifier = grader.join("logs/verifier");
    let tests = outputs_directory.task(attempt.task).join(TESTS);
    let run = Run {
        shell: &GRADER,
        user: configuration.grader_user.as_deref().unwrap_or(up.user()),
        variables: &configuration.grader_variables,
    };
    up.put(&tests, GIVEN_TESTS, run.user)
        .and_then(|()| up.put(&verifier, VERIFIER, run.user))
        .and_then(|()| up.run(&run, &grader, limits))
        .map(|ending| {
            up.copy_out_of(VERIFIER, &verifier);
            match ending {
                Ending::TimedOut => AttemptResult::TimedOut,
                Ending::Exited(_) if reward::completed(&verifier) => AttemptResult::Completed,
                Ending::Exited(_) => AttemptResult::NotCompleted,
            }
        })
}
