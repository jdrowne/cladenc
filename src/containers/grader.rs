use std::path::Path;

use super::ContainerError;
use super::container::{Shell, Up};
use super::program::{Ending, Limits};
use crate::genealogy::AttemptResult;
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "/var/lib/cladenc/task";
const GRADER: &str = "/var/lib/cladenc/task/grader";

pub(super) fn run(
    outputs_directory: &OutputsDirectory,
    up: &Up,
    task: &str,
    running: &Path,
    limits: Limits<'_>,
) -> Result<AttemptResult, ContainerError> {
    up.copy_into(&outputs_directory.task(task), TASK)?;
    let shell = Shell {
        command: GRADER,
        environment: &[("CLADENC_TASK", TASK)],
    };
    Ok(match up.run(&shell, &running.join("grader"), limits)? {
        Ending::Exited(0) => AttemptResult::Completed,
        Ending::Exited(_) => AttemptResult::NotCompleted,
        Ending::TimedOut => AttemptResult::TimedOut,
    })
}
