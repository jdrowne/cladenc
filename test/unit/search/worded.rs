use std::error::Error;
use std::io;
use std::path::PathBuf;

use cladenc::job::JobError;
use cladenc::journal::{JournalError, Started};
use cladenc::search::SearchError;

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn a_run_directory_that_cannot_be_made_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(SearchError::Io(io::Error::from_raw_os_error(17))),
        "the run directory cannot be made: File exists (os error 17)"
    );
}

#[test]
fn a_run_directory_that_cannot_be_used_is_worded_with_the_journal_s_reason() {
    assert_eq!(
        worded(SearchError::Unusable(JournalError::Locked)),
        "the run directory cannot be used: another cladenc is using it"
    );
}

#[test]
fn a_key_that_cannot_be_read_is_worded_with_its_file_and_what_the_file_has_to_hold() {
    assert_eq!(
        worded(SearchError::Key(PathBuf::from("/keys/model"))),
        "the key of the model cannot be read: /keys/model is not a file that holds a key on one \
         line and nothing else"
    );
}

#[test]
fn a_journal_that_could_not_be_written_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(SearchError::Journal(JournalError::Io(
            io::Error::from_raw_os_error(28)
        ))),
        "the search stopped before its end: its journal could not be written: \
         No space left on device (os error 28)"
    );
}

#[test]
fn a_search_that_was_asked_to_stop_is_worded() {
    assert_eq!(
        worded(SearchError::Asked),
        "the search stopped before its end: it was asked to stop"
    );
}

#[test]
fn a_search_the_build_of_its_progenitor_stopped_says_how_the_build_ended() {
    assert_eq!(
        worded(SearchError::Build(JobError::Exit(2))),
        "the search stopped before its end: the build of the progenitor exited with status 2"
    );
}

#[test]
fn a_search_whose_progenitor_took_too_long_to_build_says_so() {
    assert_eq!(
        worded(SearchError::Build(JobError::TimeLimit)),
        "the search stopped before its end: \
         the build of the progenitor ran past its time limit"
    );
}

#[test]
fn a_search_an_evaluation_stopped_names_the_harness_and_the_task() {
    assert_eq!(
        worded(SearchError::Job {
            job: Started::Evaluation {
                harness: 0,
                task: "t1".to_owned()
            },
            error: JobError::Exit(2)
        }),
        "the search stopped before its end: \
         the evaluation of harness 0 on task t1 exited with status 2"
    );
}

#[test]
fn a_search_a_procreation_stopped_names_the_parent_and_the_child() {
    assert_eq!(
        worded(SearchError::Job {
            job: Started::Procreation {
                child: 3,
                parent: 0
            },
            error: JobError::Signal(9)
        }),
        "the search stopped before its end: \
         the procreation of harness 3 by harness 0 was ended by signal 9"
    );
}
