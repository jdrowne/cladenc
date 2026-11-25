use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::container::{Bringing, Run, Shell, Up};
use super::docker::Host;
use super::health_check::{self, Backoff};
use super::program::{Ending, Limits};
use super::{
    ARTIFACTS, Checked, ContainerError, HEALTH_CHECK, Retry, Trial, VerifierImage, artifacts,
    harbor, reward,
};
use crate::benchmark::TESTS;
use crate::genealogy::TrialResult;

const VERIFIER: Shell<'static> = Shell {
    command: harbor::VERIFIER,
    environment: &[],
};

pub(super) struct Verifying<'a> {
    pub(super) host: Host<'a>,
    pub(super) trial: &'a Trial<'a>,
    pub(super) running: &'a Path,
    pub(super) stop: &'a AtomicBool,
    pub(super) max_retries: u64,
    pub(super) retrying: &'a dyn Fn(Retry),
}

impl<'a> Verifying<'a> {
    pub(super) fn run(&self, up: Up<'_>) -> Result<TrialResult, ContainerError> {
        match &self.trial.environment.verifier {
            None => self.run_in(&up, true),
            Some(image) => {
                drop(up);
                self.run_apart(image)
            }
        }
    }

    fn run_apart(&self, image: &VerifierImage) -> Result<TrialResult, ContainerError> {
        let configuration = &self.trial.environment.configuration;
        let up = self.brought_up(image)?;
        let user = configuration.verifier_user.as_deref().unwrap_or(up.user());
        let paths = self.running.join(ARTIFACTS);
        harbor::link_logs(&up)
            .and_then(|()| artifacts::give(&up, &configuration.artifacts, &paths, user))?;
        self.run_in(&up, !image.holds_verifier)
    }

    fn brought_up(&self, image: &VerifierImage) -> Result<Up<'a>, ContainerError> {
        let configuration = &self.trial.environment.configuration;
        let verifier = self.running.join("verifier");
        let bringing = Bringing::of_the_verifier(&image.name, configuration);
        let health_check = configuration
            .verifier_environment
            .as_ref()
            .and_then(|own| own.health_check.as_ref());
        let kept = verifier.join(HEALTH_CHECK);
        let backoff = Backoff {
            checked: Checked::Verifier,
            first_wait: health_check.map_or(Duration::ZERO, |health_check| health_check.interval),
            max_retries: self.max_retries,
            stop: self.stop,
        };
        let healthy = health_check::retried(&backoff, self.retrying, || {
            let up = Up::start(self.host, &bringing, &verifier)?;
            health_check::passes_where_given(&up, health_check, &kept, self.stop)
                .map(|passes| passes.then_some(up))
        })?;
        healthy.ok_or(ContainerError::VerifierHealthCheck {
            tries: self.max_retries.saturating_add(1),
            kept,
        })
    }

    fn run_in(&self, up: &Up<'_>, tests_are_given: bool) -> Result<TrialResult, ContainerError> {
        let configuration = &self.trial.environment.configuration;
        let limits = Limits::within(configuration.verifier_time_limit, self.stop);
        let verifier = self.running.join("verifier");
        let logs = verifier.join("logs/verifier");
        let tests = self
            .host
            .outputs_directory
            .task(self.trial.task)
            .join(TESTS);
        let run = Run {
            shell: &VERIFIER,
            user: configuration.verifier_user.as_deref().unwrap_or(up.user()),
            variables: &configuration.verifier_variables,
        };
        let given = if tests_are_given {
            harbor::give_tests(up, &tests, run.user)
        } else {
            Ok(())
        };
        given
            .and_then(|()| harbor::empty_verifier(up, run.user))
            .and_then(|()| up.run(&run, &verifier, limits))
            .map(|ending| {
                up.copy_out_of(harbor::VERIFIER_LOGS, &logs);
                match ending {
                    Ending::TimedOut => TrialResult::TimedOut,
                    Ending::Exited(_) if reward::completed(up.files(), &logs) => {
                        TrialResult::Completed
                    }
                    Ending::Exited(_) => TrialResult::NotCompleted,
                }
            })
    }
}
