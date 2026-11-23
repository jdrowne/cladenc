use crate::file_system::tests::double::Memory;
use crate::journal::{self, Journal, JournalError, Record};

const FIRST: &str = r#"{"record":"procreation_started","child":1,"progenitor":0}"#;
const SECOND: &str = r#"{"record":"procreation_started","child":2,"progenitor":0}"#;

fn holding(text: &str) -> Memory {
    let directory = Memory::default();
    directory
        .write(journal::path(directory.path()), text)
        .unwrap();
    directory
}

fn after_appending_the_second(directory: &Memory) -> String {
    Journal::open_existing(directory, directory.path())
        .unwrap()
        .append(&Record::ProcreationStarted {
            child: 2,
            progenitor: 0,
        })
        .unwrap();
    directory
        .read_to_string(journal::path(directory.path()))
        .unwrap()
}

#[test]
fn opening_an_existing_journal_where_there_is_none_is_an_error() {
    let directory = Memory::default();
    assert!(matches!(
        Journal::open_existing(&directory, directory.path()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn opening_an_existing_journal_where_there_is_none_creates_no_file() {
    let directory = Memory::default();
    let _refused = Journal::open_existing(&directory, directory.path());
    assert!(!directory.exists(journal::path(directory.path())));
}

#[test]
fn an_existing_journal_that_is_open_is_locked() {
    let directory = holding("");
    let _first = Journal::open_existing(&directory, directory.path()).unwrap();
    assert!(matches!(
        Journal::open_existing(&directory, directory.path()),
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
