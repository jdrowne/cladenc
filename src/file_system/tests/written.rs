use std::io::ErrorKind;
use std::path::Path;

use crate::file_system::Kind;
use crate::file_system::tests::double::Memory;

#[test]
fn a_file_that_is_written_again_holds_only_what_was_written_last() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written first\n").unwrap();
    files.write(&file, b"last\n").unwrap();
    assert_eq!(files.read(&file).unwrap(), b"last\n");
}

#[test]
fn a_file_cannot_be_written_in_a_directory_that_is_not_there() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files.write(&missing.join("file"), b"").unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_file_cannot_be_written_where_a_directory_is() {
    let directory = Memory::default();
    let files = directory.shared();
    assert_eq!(
        files.write(directory.path(), b"").unwrap_err().kind(),
        ErrorKind::IsADirectory
    );
}

#[test]
fn a_file_written_through_a_symbolic_link_is_written_where_the_link_leads() {
    let directory = Memory::default();
    let files = directory.shared();
    let link = directory.path().join("link");
    files.link(Path::new("target"), &link).unwrap();
    files.write(&link, b"written\n").unwrap();
    assert_eq!(
        files.read(&directory.path().join("target")).unwrap(),
        b"written\n"
    );
}

#[test]
fn a_directory_cannot_be_made_where_something_is() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    assert_eq!(
        files.make_directory(&made).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
}

#[test]
fn a_directory_cannot_be_made_where_a_symbolic_link_that_leads_nowhere_is() {
    let directory = Memory::default();
    let files = directory.shared();
    let link = directory.path().join("link");
    files.link(Path::new("missing"), &link).unwrap();
    assert_eq!(
        files.make_directory(&link).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
}

#[test]
fn a_directory_cannot_be_made_in_a_directory_that_is_not_there() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files
            .make_directory(&missing.join("made"))
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn directories_are_made_with_those_above_them() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("above").join("made");
    files.make_directories(&made).unwrap();
    assert_eq!(files.found(&made).unwrap().kind, Kind::Directory);
}

#[test]
fn directories_that_are_there_are_made_again_without_an_error() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("above").join("made");
    files.make_directories(&made).unwrap();
    assert!(files.make_directories(&made).is_ok());
}

#[test]
fn directories_that_are_made_again_keep_what_they_hold() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directories(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    files.make_directories(&made).unwrap();
    assert_eq!(files.read(&made.join("file")).unwrap(), b"written\n");
}

#[test]
fn directories_cannot_be_made_below_a_file() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert!(files.make_directories(&file.join("below")).is_err());
}

#[test]
fn a_symbolic_link_cannot_be_made_where_something_is() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(
        files.link(Path::new("target"), &file).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
}

#[test]
fn a_symbolic_link_cannot_be_made_in_a_directory_that_is_not_there() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files
            .link(Path::new("target"), &missing.join("link"))
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_copy_of_a_file_holds_what_the_file_holds() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.copy(&from, &to).unwrap();
    assert_eq!(files.read(&to).unwrap(), b"written\n");
}

#[test]
fn a_copy_of_a_file_has_its_permission_bits() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.set_mode(&from, 0o750).unwrap();
    files.copy(&from, &to).unwrap();
    assert_eq!(files.found(&to).unwrap().mode, 0o750);
}

#[test]
fn a_copy_takes_the_place_of_a_file_that_is_there() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"copied\n").unwrap();
    files.write(&to, b"there before\n").unwrap();
    files.copy(&from, &to).unwrap();
    assert_eq!(files.read(&to).unwrap(), b"copied\n");
}

#[test]
fn a_copy_leaves_the_file_it_is_of() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.copy(&from, &to).unwrap();
    assert_eq!(files.read(&from).unwrap(), b"written\n");
}

#[test]
fn a_copy_through_a_symbolic_link_is_a_file_of_its_own() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    let to = directory.path().join("to");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.copy(&link, &to).unwrap();
    assert_eq!(files.found(&to).unwrap().kind, Kind::File);
}

#[test]
fn a_file_that_is_not_there_cannot_be_copied() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    assert_eq!(
        files.copy(&from, &to).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn the_permission_bits_of_what_is_not_there_cannot_be_set() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files.set_mode(&missing, 0o640).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn the_permission_bits_are_set_on_what_a_symbolic_link_leads_to() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.set_mode(&link, 0o600).unwrap();
    assert_eq!(files.found(&file).unwrap().mode, 0o600);
}

#[test]
fn a_file_its_owner_may_not_read_cannot_be_copied() {
    let directory = Memory::default();
    let files = directory.shared();
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.set_mode(&from, 0o000).unwrap();
    assert_eq!(
        files.copy(&from, &to).unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
}
