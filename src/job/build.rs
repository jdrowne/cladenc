use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use super::container::{Container, Mount};
use super::program::{Ending, deadline};
use super::{HARNESS, JobError, Launch};
use crate::genealogy::{HarnessId, ProcreationOutcome};
use crate::workspace::{Workspace, holds_only_harness_files};

struct Build<'a> {
    image: &'a str,
    command: &'a str,
    harness: HarnessId,
    source: &'a Path,
}

pub fn build_progenitor(
    workspace: &Workspace,
    launch: &Launch<'_>,
    time_limit: u64,
    stop: &AtomicBool,
) -> Result<(), JobError> {
    let Some(command) = launch.build_command else {
        return Ok(());
    };
    let source = workspace.harness(0);
    let build = Build {
        image: launch.image,
        command,
        harness: 0,
        source: &source,
    };
    match build.run(workspace, deadline(time_limit), stop)? {
        Ending::Exited(0) if holds_only_harness_files(&source).unwrap_or(false) => Ok(()),
        Ending::Exited(0) => Err(JobError::OtherFiles),
        Ending::Exited(status) => Err(JobError::Exit(status)),
        Ending::TimedOut => Err(JobError::TimeLimit),
    }
}

pub(super) fn build_child(
    workspace: &Workspace,
    launch: &Launch<'_>,
    child: HarnessId,
    source: &Path,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<ProcreationOutcome, JobError> {
    let Some(command) = launch.build_command else {
        return Ok(ProcreationOutcome::Created);
    };
    let build = Build {
        image: launch.image,
        command,
        harness: child,
        source,
    };
    Ok(match build.run(workspace, deadline, stop)? {
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
        workspace: &Workspace,
        deadline: Instant,
        stop: &AtomicBool,
    ) -> Result<Ending, JobError> {
        let kept = workspace.start_build(self.harness).map_err(JobError::Io)?;
        let container = Container {
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
        container.run(&kept, deadline, stop)
    }
}
