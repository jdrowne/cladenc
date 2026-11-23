use std::io::ErrorKind;
use std::path::Path;

use crate::file_system::tests::double::Memory;

#[test]
fn a_file_that_is_removed_is_no_longer_found() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    files.remove_file(&file).unwrap();
    assert_eq!(files.found(&file).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_directory_cannot_be_removed_as_a_file() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    assert!(files.remove_file(&made).is_err());
}

#[test]
fn a_symbolic_link_that_is_removed_is_no_longer_found() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.remove_file(&link).unwrap();
    assert_eq!(files.found(&link).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_symbolic_link_that_is_removed_leaves_what_it_leads_to() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.remove_file(&link).unwrap();
    assert_eq!(files.read(&file).unwrap(), b"written\n");
}

#[test]
fn a_file_that_is_not_there_cannot_be_removed() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files.remove_file(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_directory_that_holds_nothing_is_removed() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    files.remove_directory(&made).unwrap();
    assert_eq!(files.found(&made).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_directory_that_holds_something_cannot_be_removed_alone() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    assert!(files.remove_directory(&made).is_err());
}

#[test]
fn a_directory_that_could_not_be_removed_alone_keeps_what_it_holds() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    let _ = files.remove_directory(&made);
    assert_eq!(files.read(&made.join("file")).unwrap(), b"written\n");
}

#[test]
fn a_file_cannot_be_removed_as_a_directory() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(
        files.remove_directory(&file).unwrap_err().kind(),
        ErrorKind::NotADirectory
    );
}

#[test]
fn a_directory_that_is_not_there_cannot_be_removed() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files.remove_directory(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_tree_that_is_removed_is_no_longer_found() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    files.make_directories(&made.join("below")).unwrap();
    files
        .write(&made.join("below").join("file"), b"written\n")
        .unwrap();
    files.remove_tree(&made).unwrap();
    assert_eq!(files.found(&made).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_tree_that_is_removed_leaves_what_is_beside_it() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    let beside = directory.path().join("made-beside");
    files.make_directory(&made).unwrap();
    files.write(&beside, b"kept\n").unwrap();
    files.remove_tree(&made).unwrap();
    assert_eq!(files.read(&beside).unwrap(), b"kept\n");
}

#[test]
fn a_tree_that_is_not_there_cannot_be_removed() {
    let directory = Memory::default();
    let files = directory.shared();
    let missing = directory.path().join("missing");
    assert_eq!(
        files.remove_tree(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_symbolic_link_to_a_directory_removed_as_a_tree_leaves_what_the_directory_holds() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    let link = directory.path().join("link");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    files.link(Path::new("made"), &link).unwrap();
    files.remove_tree(&link).unwrap();
    assert_eq!(files.read(&made.join("file")).unwrap(), b"written\n");
}

#[test]
fn a_symbolic_link_to_a_directory_removed_as_a_tree_is_no_longer_found() {
    let directory = Memory::default();
    let files = directory.shared();
    let made = directory.path().join("made");
    let link = directory.path().join("link");
    files.make_directory(&made).unwrap();
    files.link(Path::new("made"), &link).unwrap();
    files.remove_tree(&link).unwrap();
    assert_eq!(files.found(&link).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_file_cannot_be_removed_as_a_tree() {
    let directory = Memory::default();
    let files = directory.shared();
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert!(files.remove_tree(&file).is_err());
}
