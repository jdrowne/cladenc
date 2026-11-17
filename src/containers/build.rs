use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::container::{Container, Mount};
use super::program::{Ending, Limits};
use super::{ContainerError, HARNESS, Launch};
use crate::genealogy::{HarnessId, ProcreationOutcome};
use crate::run_directory::{RunDirectory, holds_only_harness_files};

struct Build<'a> {
    image: &'a str,
    command: &'a str,
    harness: HarnessId,
    source: &'a Path,
}

pub fn build_primogenitor(
    run_directory: &RunDirectory,
    launch: &Launch<'_>,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let Some(command) = launch.build_command else {
        return Ok(());
    };
    let source = run_directory.harness(0);
    let build = Build {
        image: launch.image,
        command,
        harness: 0,
        source: &source,
    };
    let limits = Limits::starting_now(launch.procreation_timeout, stop);
    match build.run(run_directory, limits)? {
        Ending::Exited(0) if holds_only_harness_files(&source).unwrap_or(false) => Ok(()),
        Ending::Exited(0) => Err(ContainerError::OtherFiles),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}

pub(super) fn build_child(
    run_directory: &RunDirectory,
    launch: &Launch<'_>,
    child: HarnessId,
    source: &Path,
    limits: Limits<'_>,
) -> Result<ProcreationOutcome, ContainerError> {
    let Some(command) = launch.build_command else {
        return Ok(ProcreationOutcome::Created);
    };
    let build = Build {
        image: launch.image,
        command,
        harness: child,
        source,
    };
    Ok(match build.run(run_directory, limits)? {
        Ending::Exited(0) if holds_only_harness_files(source).unwrap_or(false) => {
            ProcreationOutcome::Created
        }
        Ending::Exited(_) => ProcreationOutcome::BuildFailed,
        Ending::TimedOut => ProcreationOutcome::TimedOut,
    })
}

impl Build<'_> {
    fn run(
        &self,
        run_directory: &RunDirectory,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let kept = run_directory
            .start_build(self.harness)
            .map_err(ContainerError::Io)?;
        let container = Container {
            run_directory,
            image: self.image,
            command: self.command,
            mounts: &[Mount {
                from: self.source,
                to: HARNESS,
                read_only: false,
            }],
            working_directory: HARNESS,
            environment: &[("CLADENC_HARNESS", HARNESS)],
        };
        container.run(&kept, limits)
    }
}
