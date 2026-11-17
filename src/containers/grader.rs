use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use super::container::{Container, Mount};
use super::program::Ending;
use super::{ContainerError, WORK};
use crate::genealogy::AttemptResult;
use crate::workspace::Workspace;

const TASK: &str = "/var/lib/cladenc/task";
const GRADER: &str = "/var/lib/cladenc/task/grader";

pub(super) fn run(
    workspace: &Workspace,
    image: &str,
    task: &str,
    running: &Path,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<AttemptResult, ContainerError> {
    let work = running.join("work");
    let task = workspace.task(task);
    let container = Container {
        workspace,
        image,
        command: GRADER,
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
        match container.run(&running.join("grader"), deadline, stop)? {
            Ending::Exited(0) => AttemptResult::Completed,
            Ending::Exited(_) => AttemptResult::NotCompleted,
            Ending::TimedOut => AttemptResult::TimedOut,
        },
    )
}
