use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::container::{Container, Mount, Shell};
use super::program::{Ending, Limits};
use super::{ContainerError, HARNESS, Launch};
use crate::genealogy::{CreationOutcome, HarnessId};
use crate::outputs_directory::{OutputsDirectory, holds_only_harness_files};

struct Build<'a> {
    image: &'a str,
    command: &'a str,
    harness: HarnessId,
    source: &'a Path,
}

pub fn build_primogenitor(
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    let Some(command) = launch.build_command else {
        return Ok(());
    };
    let source = outputs_directory.harness(0);
    let build = Build {
        image: launch.image,
        command,
        harness: 0,
        source: &source,
    };
    let limits = Limits::starting_now(launch.procreation_timeout, stop);
    match build.run(outputs_directory, limits)? {
        Ending::Exited(0) if holds_only_harness_files(&source).unwrap_or(false) => Ok(()),
        Ending::Exited(0) => Err(ContainerError::OtherFiles),
        Ending::Exited(status) => Err(ContainerError::Exit(status)),
        Ending::TimedOut => Err(ContainerError::TimeLimit),
    }
}

pub(super) fn build_child(
    outputs_directory: &OutputsDirectory,
    launch: &Launch<'_>,
    child: HarnessId,
    source: &Path,
    limits: Limits<'_>,
) -> Result<CreationOutcome, ContainerError> {
    let Some(command) = launch.build_command else {
        return Ok(CreationOutcome::Created);
    };
    let build = Build {
        image: launch.image,
        command,
        harness: child,
        source,
    };
    Ok(match build.run(outputs_directory, limits)? {
        Ending::Exited(0) if holds_only_harness_files(source).unwrap_or(false) => {
            CreationOutcome::Created
        }
        Ending::Exited(_) => CreationOutcome::BuildFailed,
        Ending::TimedOut => CreationOutcome::TimedOut,
    })
}

impl Build<'_> {
    fn run(
        &self,
        outputs_directory: &OutputsDirectory,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let kept = outputs_directory
            .start_build(self.harness)
            .map_err(ContainerError::Io)?;
        let container = Container {
            outputs_directory,
            image: self.image,
            mounts: &[Mount {
                from: self.source,
                to: HARNESS,
                read_only: false,
            }],
            working_directory: HARNESS,
        };
        let shell = Shell {
            command: self.command,
            environment: &[("CLADENC_HARNESS", HARNESS)],
        };
        container.run(&shell, &kept, limits)
    }
}
