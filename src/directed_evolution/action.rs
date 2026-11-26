use std::io;

use super::Shared;
use crate::containers::{self, ContainerError, Creation, Host, Retry, Trial, Trialled};
use crate::genealogy::{Experiment, HarnessId};
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;

pub(super) enum Told {
    Ended(Ended),
    Retrying {
        harness: HarnessId,
        task: String,
        retry: Retry,
    },
}

pub(super) enum Ended {
    Diversified {
        child: HarnessId,
        progenitor: HarnessId,
        creation: Creation,
    },
    Established {
        harness: HarnessId,
        experiment: Experiment,
        creation: Creation,
    },
    Trialled {
        harness: HarnessId,
        task: String,
        trialled: Trialled,
    },
    Stopped,
    Failed {
        action: Started,
        error: ContainerError,
    },
}

pub(super) fn carry_out(shared: &Shared<'_>, action: Started, again: bool) -> Ended {
    let outputs_directory = &shared.outputs_directory;
    let host = Host {
        docker: shared.docker,
        outputs_directory,
    };
    let cleared = if again {
        discard(outputs_directory, &action)
    } else {
        Ok(())
    };
    let cleared = cleared.map_err(ContainerError::Io);
    let result = match action.clone() {
        Started::Diversification { child, progenitor } => cleared
            .and_then(|()| {
                containers::run_diversification(
                    host,
                    &shared.launch,
                    child,
                    progenitor,
                    &shared.halt,
                )
            })
            .map(|creation| Ended::Diversified {
                child,
                progenitor,
                creation,
            }),
        Started::Baseline {
            harness,
            child,
            progenitor,
        } => {
            let experiment = Experiment { child, progenitor };
            cleared
                .and_then(|()| {
                    containers::run_baseline(
                        host,
                        &shared.launch,
                        harness,
                        experiment,
                        &shared.halt,
                    )
                })
                .map(|creation| Ended::Established {
                    harness,
                    experiment,
                    creation,
                })
        }
        Started::Trial { harness, task } => cleared
            .and_then(|()| {
                let environment = shared.environments.get(&task).cloned().unwrap_or_default();
                let trial = Trial {
                    harness,
                    task: &task,
                    environment: &environment,
                };
                let retrying = |retry: Retry| {
                    let _ = shared.sender.send(Told::Retrying {
                        harness,
                        task: task.clone(),
                        retry,
                    });
                };
                containers::run_trial(host, &shared.launch, trial, &shared.halt, &retrying)
            })
            .map(|trialled| Ended::Trialled {
                harness,
                task,
                trialled,
            }),
    };
    match result {
        Ok(ended) => ended,
        Err(ContainerError::Stopped) => Ended::Stopped,
        Err(error) => Ended::Failed { action, error },
    }
}

pub(super) fn discard(outputs_directory: &OutputsDirectory, action: &Started) -> io::Result<()> {
    match action {
        Started::Diversification { child, .. } => outputs_directory.discard_diversification(*child),
        Started::Baseline { harness, .. } => outputs_directory.discard_baseline(*harness),
        Started::Trial { harness, task } => outputs_directory.discard_trial(*harness, task),
    }
}
