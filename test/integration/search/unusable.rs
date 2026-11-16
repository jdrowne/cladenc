use std::fs;
use std::io;
use std::sync::atomic::AtomicBool;

use cladenc::journal::{self, Journal, JournalError};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn resumed(directory: &TempDir) -> Result<Found, SearchError> {
    search::resume(directory.path(), &AtomicBool::new(false), &mut io::sink())
}

fn holding(journal: &str) -> TempDir {
    let directory = test_inputs::temporary_directory();
    fs::write(journal::path(directory.path()), journal).unwrap();
    directory
}

#[test]
fn a_run_directory_that_does_not_exist_cannot_be_resumed() {
    let directory = test_inputs::temporary_directory();
    let missing = directory.path().join("missing");
    assert!(matches!(
        search::resume(&missing, &AtomicBool::new(false), &mut io::sink()),
        Err(SearchError::Unusable(JournalError::Io(_)))
    ));
}

#[test]
fn a_directory_that_holds_no_journal_cannot_be_resumed() {
    assert!(matches!(
        resumed(&test_inputs::temporary_directory()),
        Err(SearchError::Unusable(JournalError::Io(_)))
    ));
}

#[test]
fn a_directory_that_holds_no_journal_is_given_none() {
    let directory = test_inputs::temporary_directory();
    let _refused = resumed(&directory);
    assert!(!journal::path(directory.path()).exists());
}

#[test]
fn a_run_directory_in_use_cannot_be_resumed() {
    let directory = holding("");
    let _in_use = Journal::open(directory.path()).unwrap();
    assert!(matches!(
        resumed(&directory),
        Err(SearchError::Unusable(JournalError::Locked))
    ));
}

#[test]
fn a_run_directory_whose_journal_cannot_be_read_cannot_be_resumed() {
    assert!(matches!(
        resumed(&holding("not a record\n")),
        Err(SearchError::Unusable(JournalError::NotARecord { line: 1 }))
    ));
}
