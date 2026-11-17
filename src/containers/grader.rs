use std::path::Path;

use super::container::{Container, Mount};
use super::program::{Ending, Limits};
use super::{ContainerError, WORK};
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "/var/lib/cladenc/task";
const GRADER: &str = "/var/lib/cladenc/task/grader";

pub(super) fn run(
    outputs_directory: &OutputsDirectory,
    image: &str,
    task: &str,
    running: &Path,
    limits: Limits<'_>,
) -> Result<AttemptResult, ContainerError> {
    let work = running.join("work");
    let task = outputs_directory.task(task);
    let container = Container {
        outputs_directory,
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
    Ok(match container.run(&running.join("grader"), limits)? {
        Ending::Exited(0) => AttemptResult::Completed,
        Ending::Exited(_) => AttemptResult::NotCompleted,
        Ending::TimedOut => AttemptResult::TimedOut,
    })
}
