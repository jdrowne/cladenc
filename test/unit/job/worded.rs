use std::error::Error;
use std::io;

use cladenc::job::JobError;

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn a_job_that_was_stopped_is_worded() {
    assert_eq!(worded(JobError::Stopped), "was stopped");
}

#[test]
fn a_job_that_could_not_be_carried_out_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(JobError::Io(io::Error::from_raw_os_error(2))),
        "could not be carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_program_that_exited_with_another_status_is_worded() {
    assert_eq!(worded(JobError::Exit(2)), "exited with status 2");
}

#[test]
fn a_program_ended_by_a_signal_is_worded() {
    assert_eq!(worded(JobError::Signal(9)), "was ended by signal 9");
}
