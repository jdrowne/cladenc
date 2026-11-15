use std::fs;
use std::path::Path;

use cladenc::journal::{self, Journal, JournalError, Record};
use tempfile::TempDir;

const FIRST: &str = r#"{"record":"expansion_started","child":1,"parent":0}"#;
const SECOND: &str = r#"{"record":"expansion_started","child":2,"parent":0}"#;

fn started(child: u64) -> Record {
    Record::ExpansionStarted { child, parent: 0 }
}

fn holding(text: &str) -> TempDir {
    let run_directory = tempfile::tempdir().unwrap();
    fs::write(journal::path(run_directory.path()), text).unwrap();
    run_directory
}

fn after_appending(run_directory: &TempDir, record: &Record) -> String {
    Journal::open(run_directory.path())
        .unwrap()
        .append(record)
        .unwrap();
    fs::read_to_string(journal::path(run_directory.path())).unwrap()
}

#[test]
fn the_journal_is_journal_jsonl_in_the_run_directory() {
    assert_eq!(
        journal::path(Path::new("/run")),
        Path::new("/run/journal.jsonl")
    );
}

#[test]
fn opening_a_journal_that_does_not_exist_creates_its_file() {
    let run_directory = tempfile::tempdir().unwrap();
    let _journal = Journal::open(run_directory.path()).unwrap();
    assert!(journal::path(run_directory.path()).is_file());
}

#[test]
fn a_journal_cannot_be_opened_in_a_directory_that_does_not_exist() {
    let run_directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        Journal::open(&run_directory.path().join("missing")),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_record_is_appended_as_its_line_and_a_newline() {
    assert_eq!(
        after_appending(&holding(""), &started(1)),
        format!("{FIRST}\n")
    );
}

#[test]
fn a_record_is_appended_after_the_records_the_journal_holds() {
    assert_eq!(
        after_appending(&holding(&format!("{FIRST}\n")), &started(2)),
        format!("{FIRST}\n{SECOND}\n")
    );
}

#[test]
fn two_records_appended_to_one_journal_are_kept_in_their_order() {
    let run_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(run_directory.path()).unwrap();
    journal.append(&started(1)).unwrap();
    journal.append(&started(2)).unwrap();
    assert_eq!(
        fs::read_to_string(journal::path(run_directory.path())).unwrap(),
        format!("{FIRST}\n{SECOND}\n")
    );
}

#[test]
fn a_cut_short_last_line_is_gone_after_the_next_record_is_appended() {
    let cut_short = holding(&format!("{FIRST}\n{{\"record\":\"expansion_fin"));
    assert_eq!(
        after_appending(&cut_short, &started(2)),
        format!("{FIRST}\n{SECOND}\n")
    );
}

#[test]
fn a_cut_short_first_line_is_gone_after_the_next_record_is_appended() {
    assert_eq!(
        after_appending(&holding("{\"record\":\"ru"), &started(1)),
        format!("{FIRST}\n")
    );
}
