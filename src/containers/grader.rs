use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::container::{Bringing, Run, Shell, Up};
use super::program::{Ending, Limits};
use super::{
    ARTIFACTS, Attempt, ContainerError, GraderImage, HEALTH_CHECK, artifacts, health_check, reward,
};
use crate::benchmark::TESTS;
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const GIVEN_TESTS: &str = "/tests";
const VERIFIER: &str = "/logs/verifier";
const GRADER: Shell<'static> = Shell {
    command: "/tests/test.sh",
    environment: &[],
};

pub(super) struct Grading<'a> {
    pub(super) outputs_directory: &'a OutputsDirectory,
    pub(super) attempt: &'a Attempt<'a>,
    pub(super) running: &'a Path,
    pub(super) stop: &'a AtomicBool,
}

impl Grading<'_> {
    pub(super) fn run(&self, up: Up) -> Result<AttemptResult, ContainerError> {
        match &self.attempt.environment.grader {
            None => self.run_in(&up, true),
            Some(image) => {
                drop(up);
                self.run_apart(image)
            }
        }
    }

    fn run_apart(&self, image: &GraderImage) -> Result<AttemptResult, ContainerError> {
        let configuration = &self.attempt.environment.configuration;
        let grader = self.running.join("grader");
        let bringing = Bringing::of_the_grader(&image.name, configuration);
        let up = Up::start(self.outputs_directory, &bringing, &grader)?;
        let health_check = configuration
            .grader_environment
            .as_ref()
            .and_then(|own| own.health_check.as_ref());
        let kept = grader.join(HEALTH_CHECK);
        if !health_check::passes_where_given(&up, health_check, &kept, self.stop)? {
            return Ok(AttemptResult::NotCompleted);
        }
        let user = configuration.grader_user.as_deref().unwrap_or(up.user());
        artifacts::give(
            &up,
            &configuration.artifacts,
            &self.running.join(ARTIFACTS),
            user,
        )?;
        self.run_in(&up, !image.holds_grader)
    }

    fn run_in(&self, up: &Up, tests_are_given: bool) -> Result<AttemptResult, ContainerError> {
        let configuration = &self.attempt.environment.configuration;
        let limits = Limits::within(configuration.grader_time_limit, self.stop);
        let grader = self.running.join("grader");
        let verifier = grader.join("logs/verifier");
        let tests = self.outputs_directory.task(self.attempt.task).join(TESTS);
        let run = Run {
            shell: &GRADER,
            user: configuration.grader_user.as_deref().unwrap_or(up.user()),
            variables: &configuration.grader_variables,
        };
        let given = if tests_are_given {
            up.put(&tests, GIVEN_TESTS, run.user)
        } else {
            Ok(())
        };
        given
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
}
