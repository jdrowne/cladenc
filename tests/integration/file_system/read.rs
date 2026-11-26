use std::io::ErrorKind;
use std::path::Path;

use cladenc::file_system::{Disk, FileSystem};

#[test]
fn a_directory_is_listed_by_the_names_of_what_it_holds() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    files.write(&directory.path().join("a"), b"").unwrap();
    files.make_directory(&directory.path().join("b")).unwrap();
    files
        .link(Path::new("a"), &directory.path().join("c"))
        .unwrap();
    let mut listed = files.list(directory.path()).unwrap();
    listed.sort();
    assert_eq!(listed, ["a", "b", "c"]);
}

#[test]
fn a_directory_is_listed_without_what_the_directories_in_it_hold() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    assert_eq!(files.list(directory.path()).unwrap(), ["made"]);
}

#[test]
fn a_directory_that_holds_nothing_is_listed_as_empty() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    assert!(files.list(directory.path()).unwrap().is_empty());
}

#[test]
fn a_directory_is_listed_through_a_symbolic_link() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    let link = directory.path().join("link");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    files.link(Path::new("made"), &link).unwrap();
    assert_eq!(files.list(&link).unwrap(), ["file"]);
}

#[test]
fn a_file_cannot_be_listed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(
        files.list(&file).unwrap_err().kind(),
        ErrorKind::NotADirectory
    );
}

#[test]
fn a_directory_that_is_not_there_cannot_be_listed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.list(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_file_is_read_as_the_bytes_it_holds() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, &[0xff, 0xfe]).unwrap();
    assert_eq!(files.read(&file).unwrap(), [0xff, 0xfe]);
}

#[test]
fn a_file_is_read_through_a_symbolic_link() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    assert_eq!(files.read(&link).unwrap(), b"written\n");
}

#[test]
fn a_file_that_is_not_there_cannot_be_read() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.read(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_directory_cannot_be_read_as_a_file() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    assert_eq!(
        files.read(directory.path()).unwrap_err().kind(),
        ErrorKind::IsADirectory
    );
}

#[test]
fn a_file_of_text_is_read_as_text() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(files.read_text(&file).unwrap(), "written\n");
}

#[test]
fn a_file_that_is_not_text_cannot_be_read_as_text() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, &[0xff, 0xfe]).unwrap();
    assert_eq!(
        files.read_text(&file).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
}

#[test]
fn a_file_of_text_that_is_not_there_cannot_be_read() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.read_text(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn the_target_of_a_symbolic_link_is_read_as_it_was_written() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let link = directory.path().join("link");
    files.link(Path::new("../a/target"), &link).unwrap();
    assert_eq!(files.target(&link).unwrap(), Path::new("../a/target"));
}

#[test]
fn a_file_has_no_target() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(
        files.target(&file).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
}

#[test]
fn what_is_not_there_has_no_target() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.target(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_file_its_owner_may_not_read_cannot_be_read() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    files.set_mode(&file, 0o000).unwrap();
    assert_eq!(
        files.read(&file).unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
}
