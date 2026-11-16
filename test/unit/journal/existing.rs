use std::fs;

use cladenc::journal::{self, Journal, JournalError, Record};
use tempfile::TempDir;

const FIRST: &str = r#"{"record":"procreation_started","child":1,"progenitor":0}"#;
const SECOND: &str = r#"{"record":"procreation_started","child":2,"progenitor":0}"#;

fn holding(text: &str) -> TempDir {
    let run_directory = tempfile::tempdir().unwrap();
    fs::write(journal::path(run_directory.path()), text).unwrap();
    run_directory
}

fn after_appending_the_second(run_directory: &TempDir) -> String {
    Journal::open_existing(run_directory.path())
        .unwrap()
        .append(&Record::ProcreationStarted {
            child: 2,
            progenitor: 0,
        })
        .unwrap();
    fs::read_to_string(journal::path(run_directory.path())).unwrap()
}

#[test]
fn opening_an_existing_journal_where_there_is_none_is_an_error() {
    let run_directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        Journal::open_existing(run_directory.path()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn opening_an_existing_journal_where_there_is_none_creates_no_file() {
    let run_directory = tempfile::tempdir().unwrap();
    let _refused = Journal::open_existing(run_directory.path());
    assert!(!journal::path(run_directory.path()).exists());
}

#[test]
fn an_existing_journal_that_is_open_is_locked() {
    let run_directory = holding("");
    let _first = Journal::open_existing(run_directory.path()).unwrap();
    assert!(matches!(
        Journal::open_existing(run_directory.path()),
        Err(JournalError::Locked)
    ));
}

#[test]
fn a_record_appended_to_an_existing_journal_goes_after_the_records_it_holds() {
    assert_eq!(
        after_appending_the_second(&holding(&format!("{FIRST}\n"))),
        format!("{FIRST}\n{SECOND}\n")
    );
}

#[test]
fn a_cut_short_last_line_of_an_existing_journal_is_gone_after_the_next_record_is_appended() {
    let cut_short = holding(&format!("{FIRST}\n{{\"record\":\"procreation_fin"));
    assert_eq!(
        after_appending_the_second(&cut_short),
        format!("{FIRST}\n{SECOND}\n")
    );
}
