use std::io::ErrorKind;
use std::path::Path;

use cladenc::file_system::{Disk, FileSystem, Kind};

#[test]
fn a_file_that_is_renamed_is_at_its_new_name() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(files.read(&to).unwrap(), b"written\n");
}

#[test]
fn a_file_that_is_renamed_is_no_longer_at_its_old_name() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(files.found(&from).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_directory_that_is_renamed_keeps_what_it_holds() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.make_directory(&from).unwrap();
    files.write(&from.join("file"), b"written\n").unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(files.read(&to.join("file")).unwrap(), b"written\n");
}

#[test]
fn a_directory_that_is_renamed_holds_nothing_at_its_old_name() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.make_directory(&from).unwrap();
    files.write(&from.join("file"), b"written\n").unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(
        files.found(&from.join("file")).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_file_renamed_onto_a_file_takes_its_place() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"moved\n").unwrap();
    files.write(&to, b"there before\n").unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(files.read(&to).unwrap(), b"moved\n");
}

#[test]
fn a_directory_renamed_onto_a_directory_that_holds_nothing_takes_its_place() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.make_directory(&from).unwrap();
    files.write(&from.join("file"), b"written\n").unwrap();
    files.make_directory(&to).unwrap();
    files.rename(&from, &to).unwrap();
    assert_eq!(files.read(&to.join("file")).unwrap(), b"written\n");
}

#[test]
fn a_directory_cannot_be_renamed_onto_a_directory_that_holds_something() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.make_directory(&from).unwrap();
    files.make_directory(&to).unwrap();
    files.write(&to.join("file"), b"written\n").unwrap();
    assert!(files.rename(&from, &to).is_err());
}

#[test]
fn a_file_cannot_be_renamed_onto_a_directory() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    files.make_directory(&to).unwrap();
    assert!(files.rename(&from, &to).is_err());
}

#[test]
fn a_directory_cannot_be_renamed_onto_a_file() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.make_directory(&from).unwrap();
    files.write(&to, b"written\n").unwrap();
    assert!(files.rename(&from, &to).is_err());
}

#[test]
fn a_directory_cannot_be_renamed_into_itself() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    files.make_directory(&from).unwrap();
    assert!(files.rename(&from, &from.join("below")).is_err());
}

#[test]
fn what_is_not_there_cannot_be_renamed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    assert_eq!(
        files.rename(&from, &to).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn what_is_renamed_into_a_directory_that_is_not_there_is_not_renamed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let from = directory.path().join("from");
    let to = directory.path().join("to");
    files.write(&from, b"written\n").unwrap();
    assert_eq!(
        files.rename(&from, &to.join("below")).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_symbolic_link_that_is_renamed_is_still_a_symbolic_link() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    let to = directory.path().join("to");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.rename(&link, &to).unwrap();
    assert_eq!(files.found(&to).unwrap().kind, Kind::SymbolicLink);
}

#[test]
fn a_symbolic_link_that_is_renamed_leaves_what_it_leads_to_where_it_is() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    let to = directory.path().join("to");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.rename(&link, &to).unwrap();
    assert_eq!(files.read(&file).unwrap(), b"written\n");
}
