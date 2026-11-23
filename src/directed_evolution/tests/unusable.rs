use std::io;
use std::sync::atomic::AtomicBool;

use crate::containers::tests::double::Double;
use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Journal, JournalError};

fn resumed(directory: &Memory) -> Result<Found, DirectedEvolutionError> {
    directed_evolution::resume(
        Double::default().on(directory).outside(),
        directory.path(),
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

fn holding(journal: &str) -> Memory {
    let directory = Memory::default();
    directory
        .write(journal::path(directory.path()), journal)
        .unwrap();
    directory
}

#[test]
fn an_outputs_directory_that_does_not_exist_cannot_be_resumed() {
    let directory = Memory::default();
    let missing = directory.path().join("missing");
    assert!(matches!(
        directed_evolution::resume(
            Double::default().on(&directory).outside(),
            &missing,
            &AtomicBool::new(false),
            &mut io::sink()
        ),
        Err(DirectedEvolutionError::Unusable(JournalError::Io(_)))
    ));
}

#[test]
fn a_directory_that_holds_no_journal_cannot_be_resumed() {
    assert!(matches!(
        resumed(&Memory::default()),
        Err(DirectedEvolutionError::Unusable(JournalError::Io(_)))
    ));
}

#[test]
fn a_directory_that_holds_no_journal_is_given_none() {
    let directory = Memory::default();
    let _refused = resumed(&directory);
    assert!(!directory.exists(journal::path(directory.path())));
}

#[test]
fn an_outputs_directory_in_use_cannot_be_resumed() {
    let directory = holding("");
    let _in_use = Journal::open(&directory, directory.path()).unwrap();
    assert!(matches!(
        resumed(&directory),
        Err(DirectedEvolutionError::Unusable(JournalError::Locked))
    ));
}

#[test]
fn an_outputs_directory_whose_journal_cannot_be_read_cannot_be_resumed() {
    assert!(matches!(
        resumed(&holding("not a record\n")),
        Err(DirectedEvolutionError::Unusable(JournalError::NotARecord {
            line: 1
        }))
    ));
}
