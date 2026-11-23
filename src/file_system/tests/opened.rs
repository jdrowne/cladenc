use std::io::ErrorKind;

use crate::file_system::Kind;
use crate::file_system::tests::double::Memory;

#[test]
fn a_file_that_is_not_there_is_made_when_it_is_opened_to_be_made() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.open(&file, true).unwrap();
    assert_eq!(files.found(&file).unwrap().kind, Kind::File);
}

#[test]
fn a_file_that_is_made_when_it_is_opened_holds_nothing() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut opened = files.open(&file, true).unwrap();
    assert!(opened.contents().unwrap().is_empty());
}

#[test]
fn a_file_that_is_not_there_is_not_opened_unless_it_is_to_be_made() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    assert_eq!(
        files.open(&file, false).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_file_that_was_not_opened_is_not_made() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let _ = files.open(&file, false);
    assert_eq!(files.found(&file).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_file_that_is_there_keeps_what_it_holds_when_it_is_opened_to_be_made() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    let mut opened = files.open(&file, true).unwrap();
    assert_eq!(opened.contents().unwrap(), b"written\n");
}

#[test]
fn a_directory_cannot_be_opened_as_a_file() {
    let directory = Memory::default();
    let files = directory.shared();
    assert!(files.open(directory.path(), false).is_err());
}

#[test]
fn an_opened_file_gives_what_it_holds() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    assert_eq!(opened.contents().unwrap(), b"written\n");
}

#[test]
fn a_line_appended_to_an_opened_file_is_at_its_end() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"one\n").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    opened.append(b"two\n").unwrap();
    assert_eq!(files.read(&file).unwrap(), b"one\ntwo\n");
}

#[test]
fn an_opened_file_that_is_cut_holds_only_its_first_bytes() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"one\ntw").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    opened.cut(4).unwrap();
    assert_eq!(files.read(&file).unwrap(), b"one\n");
}

#[test]
fn a_line_appended_after_a_cut_follows_what_was_kept() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"one\ntw").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    opened.cut(4).unwrap();
    opened.append(b"two\n").unwrap();
    assert_eq!(files.read(&file).unwrap(), b"one\ntwo\n");
}

#[test]
fn a_line_appended_after_the_file_was_read_and_cut_follows_what_was_kept() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"one\ntw").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    opened
        .contents()
        .and_then(|_| opened.cut(4))
        .and_then(|()| opened.append(b"two\n"))
        .unwrap();
    assert_eq!(files.read(&file).unwrap(), b"one\ntwo\n");
}

#[test]
fn an_opened_file_can_be_locked() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut opened = files.open(&file, true).unwrap();
    assert!(opened.lock());
}

#[test]
fn a_file_locked_through_one_opening_cannot_be_locked_through_another() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut first = files.open(&file, true).unwrap();
    let mut second = files.open(&file, true).unwrap();
    first.lock();
    assert!(!second.lock());
}

#[test]
fn a_file_can_be_locked_once_the_opening_that_locked_it_is_gone() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut first = files.open(&file, true).unwrap();
    let mut second = files.open(&file, true).unwrap();
    first.lock();
    drop(first);
    assert!(second.lock());
}

#[test]
fn a_file_stays_locked_when_it_is_locked_again_through_the_same_opening() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut opened = files.open(&file, true).unwrap();
    opened.lock();
    assert!(opened.lock());
}

#[test]
fn a_file_stays_locked_when_an_opening_that_could_not_lock_it_is_gone() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let mut first = files.open(&file, true).unwrap();
    let mut second = files.open(&file, true).unwrap();
    first.lock();
    second.lock();
    drop(second);
    let mut third = files.open(&file, true).unwrap();
    assert!(!third.lock());
}

#[test]
fn a_file_that_is_locked_can_still_be_read() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    let mut opened = files.open(&file, false).unwrap();
    opened.lock();
    assert_eq!(files.read(&file).unwrap(), b"written\n");
}
