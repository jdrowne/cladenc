use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::docker::Host;
use super::program::{Ending, Limits};
use super::sandbox::{Mount, Sandbox, Shell};
use super::{HARNESS, Launch, SandboxError};
use crate::genealogy::{CreationOutcome, HarnessId};
use crate::outputs_directory::{holds_only_harness_files, make_usable};

struct Build<'a> {
    image: &'a str,
    command: &'a str,
    harness: HarnessId,
    source: &'a Path,
}

pub fn build_primogenitor(
    host: Host<'_>,
    launch: &Launch<'_>,
    stop: &AtomicBool,
) -> Result<(), SandboxError> {
    let Some(command) = launch.build_command else {
        return Ok(());
    };
    let files = host.outputs_directory.files();
    let source = host.outputs_directory.harness(0);
    let build = Build {
        image: launch.image,
        command,
        harness: 0,
        source: &source,
    };
    let limits = Limits::starting_now(launch.diversification_timeout, stop);
    match build.run(host, limits)? {
        Ending::Exited(0) if holds_only_harness_files(files, &source).unwrap_or(false) => Ok(()),
        Ending::Exited(0) => Err(SandboxError::OtherFiles),
        Ending::Exited(status) => Err(SandboxError::Exit(status)),
        Ending::TimedOut => Err(SandboxError::TimeLimit),
    }
}

pub(super) fn build_child(
    host: Host<'_>,
    launch: &Launch<'_>,
    child: HarnessId,
    source: &Path,
    limits: Limits<'_>,
) -> Result<CreationOutcome, SandboxError> {
    let Some(command) = launch.build_command else {
        return Ok(CreationOutcome::Created);
    };
    let files = host.outputs_directory.files();
    let build = Build {
        image: launch.image,
        command,
        harness: child,
        source,
    };
    Ok(match build.run(host, limits)? {
        Ending::Exited(0) if holds_only_harness_files(files, source).unwrap_or(false) => {
            CreationOutcome::Created
        }
        Ending::Exited(_) => CreationOutcome::BuildFailed,
        Ending::TimedOut => CreationOutcome::TimedOut,
    })
}

impl Build<'_> {
    fn run(&self, host: Host<'_>, limits: Limits<'_>) -> Result<Ending, SandboxError> {
        let kept = host
            .outputs_directory
            .start_build(self.harness)
            .map_err(SandboxError::Io)?;
        let sandbox = Sandbox {
            host,
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
        sandbox.run(&shell, &kept, limits).inspect(|_| {
            make_usable(host.outputs_directory.files(), self.source);
        })
    }
}
