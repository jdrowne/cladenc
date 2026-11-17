use cladenc::journal::{Journal, JournalError};

#[test]
fn a_second_lock_on_a_journal_is_refused_while_the_first_is_held() {
    let outputs_directory = tempfile::tempdir().unwrap();
    let _first = Journal::open(outputs_directory.path()).unwrap();
    assert!(matches!(
        Journal::open(outputs_directory.path()),
        Err(JournalError::Locked)
    ));
}

#[test]
fn a_journal_can_be_locked_again_once_the_first_lock_is_let_go() {
    let outputs_directory = tempfile::tempdir().unwrap();
    drop(Journal::open(outputs_directory.path()).unwrap());
    assert!(Journal::open(outputs_directory.path()).is_ok());
}
