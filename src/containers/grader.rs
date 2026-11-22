use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::container::{Bringing, Run, Shell, Up};
use super::health_check::{self, Backoff};
use super::program::{Ending, Limits};
use super::{
    ARTIFACTS, Attempt, Checked, ContainerError, GraderImage, HEALTH_CHECK, Retry, artifacts,
    harbor, reward,
};
use crate::benchmark::TESTS;
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const GRADER: Shell<'static> = Shell {
    command: harbor::GRADER,
    environment: &[],
};

pub(super) struct Grading<'a> {
    pub(super) outputs_directory: &'a OutputsDirectory,
    pub(super) attempt: &'a Attempt<'a>,
    pub(super) running: &'a Path,
    pub(super) stop: &'a AtomicBool,
    pub(super) max_retries: u64,
    pub(super) retrying: &'a dyn Fn(Retry),
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
        let up = self.brought_up(image)?;
        let user = configuration.grader_user.as_deref().unwrap_or(up.user());
        let paths = self.running.join(ARTIFACTS);
        harbor::link_logs(&up)
            .and_then(|()| artifacts::give(&up, &configuration.artifacts, &paths, user))?;
        self.run_in(&up, !image.holds_grader)
    }

    fn brought_up(&self, image: &GraderImage) -> Result<Up, ContainerError> {
        let configuration = &self.attempt.environment.configuration;
        let grader = self.running.join("grader");
        let bringing = Bringing::of_the_grader(&image.name, configuration);
        let health_check = configuration
            .grader_environment
            .as_ref()
            .and_then(|own| own.health_check.as_ref());
        let kept = grader.join(HEALTH_CHECK);
        let backoff = Backoff {
            checked: Checked::Grader,
            first_wait: health_check.map_or(Duration::ZERO, |health_check| health_check.interval),
            max_retries: self.max_retries,
            stop: self.stop,
        };
        let healthy = health_check::retried(&backoff, self.retrying, || {
            let up = Up::start(self.outputs_directory, &bringing, &grader)?;
            health_check::passes_where_given(&up, health_check, &kept, self.stop)
                .map(|passes| passes.then_some(up))
        })?;
        healthy.ok_or(ContainerError::GraderHealthCheck {
            tries: self.max_retries.saturating_add(1),
            kept,
        })
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
            harbor::give_tests(up, &tests, run.user)
        } else {
            Ok(())
        };
        given
            .and_then(|()| harbor::empty_verifier(up, run.user))
            .and_then(|()| up.run(&run, &grader, limits))
            .map(|ending| {
                up.copy_out_of(harbor::VERIFIER, &verifier);
                match ending {
                    Ending::TimedOut => AttemptResult::TimedOut,
                    Ending::Exited(_) if reward::completed(&verifier) => AttemptResult::Completed,
                    Ending::Exited(_) => AttemptResult::NotCompleted,
                }
            })
    }
}
