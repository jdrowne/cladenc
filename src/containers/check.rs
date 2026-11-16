use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use super::container::{Container, Mount};
use super::program::Ending;
use super::{ContainerError, WORK};
use crate::genealogy::AssessmentOutcome;
use crate::workspace::Workspace;

const TASK: &str = "/var/lib/cladenc/task";
const CHECK: &str = "/var/lib/cladenc/task/check";

pub(super) fn run(
    workspace: &Workspace,
    image: &str,
    task: &str,
    running: &Path,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<AssessmentOutcome, ContainerError> {
    let work = running.join("work");
    let task = workspace.task(task);
    let container = Container {
        workspace,
        image,
        command: CHECK,
        mounts: &[
            Mount {
                from: &work,
                to: WORK,
                read_only: false,
            },
            Mount {
                from: &task,
                to: TASK,
                read_only: true,
            },
        ],
        working_directory: WORK,
        environment: &[("CLADENC_TASK", TASK)],
    };
    Ok(
        match container.run(&running.join("check"), deadline, stop)? {
            Ending::Exited(0) => AssessmentOutcome::Success,
            Ending::Exited(_) => AssessmentOutcome::Failure,
            Ending::TimedOut => AssessmentOutcome::TimedOut,
        },
    )
}
