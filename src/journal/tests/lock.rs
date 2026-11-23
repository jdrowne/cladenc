use crate::file_system::tests::double::Memory;
use crate::journal::{Journal, JournalError};

#[test]
fn a_second_lock_on_a_journal_is_refused_while_the_first_is_held() {
    let directory = Memory::default();
    let _first = Journal::open(&directory, directory.path()).unwrap();
    assert!(matches!(
        Journal::open(&directory, directory.path()),
        Err(JournalError::Locked)
    ));
}

#[test]
fn a_journal_can_be_locked_again_once_the_first_lock_is_let_go() {
    let directory = Memory::default();
    drop(Journal::open(&directory, directory.path()).unwrap());
    assert!(Journal::open(&directory, directory.path()).is_ok());
}
