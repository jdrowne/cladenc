use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use super::container::{Container, Mount};
use super::program::Ending;
use super::{JobError, WORK};
use crate::genealogy::EvaluationOutcome;

const TASK: &str = "/task";
const CHECK: &str = "/task/check";

pub(super) fn run(
    image: &str,
    task: &Path,
    job: &Path,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<EvaluationOutcome, JobError> {
    let work = job.join("work");
    let container = Container {
        image,
        command: CHECK,
        mounts: &[
            Mount {
                from: &work,
                to: WORK,
                read_only: false,
            },
            Mount {
                from: task,
                to: TASK,
                read_only: true,
            },
        ],
        working_directory: WORK,
        environment: &[("CLADENC_TASK", TASK)],
    };
    Ok(match container.run(&job.join("check"), deadline, stop)? {
        Ending::Exited(0) => EvaluationOutcome::Success,
        Ending::Exited(_) => EvaluationOutcome::Failure,
        Ending::TimedOut => EvaluationOutcome::TimedOut,
    })
}
