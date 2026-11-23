use std::io::ErrorKind;
use std::os::unix::net::UnixListener;
use std::path::Path;

use cladenc::file_system::{Disk, FileSystem, Kind};

#[test]
fn a_directory_is_found_as_a_directory() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    assert_eq!(files.found(&made).unwrap().kind, Kind::Directory);
}

#[test]
fn a_file_is_found_as_a_file() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(files.found(&file).unwrap().kind, Kind::File);
}

#[test]
fn a_symbolic_link_is_found_as_a_symbolic_link_and_not_as_what_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    assert_eq!(files.found(&link).unwrap().kind, Kind::SymbolicLink);
}

#[test]
fn a_symbolic_link_is_followed_to_what_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    assert_eq!(files.followed(&link).unwrap().kind, Kind::File);
}

#[test]
fn a_symbolic_link_to_a_directory_is_followed_to_the_directory() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    let link = directory.path().join("link");
    files.make_directory(&made).unwrap();
    files.link(Path::new("made"), &link).unwrap();
    assert_eq!(files.followed(&link).unwrap().kind, Kind::Directory);
}

#[test]
fn a_symbolic_link_that_leads_nowhere_is_found() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let link = directory.path().join("link");
    files.link(Path::new("missing"), &link).unwrap();
    assert_eq!(files.found(&link).unwrap().kind, Kind::SymbolicLink);
}

#[test]
fn a_symbolic_link_that_leads_nowhere_cannot_be_followed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let link = directory.path().join("link");
    files.link(Path::new("missing"), &link).unwrap();
    assert_eq!(
        files.followed(&link).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn what_is_not_there_is_not_found() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.found(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn what_is_not_there_cannot_be_followed() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.followed(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn what_is_below_a_file_is_not_found() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(
        files.found(&file.join("below")).unwrap_err().kind(),
        ErrorKind::NotADirectory
    );
}

#[test]
fn a_file_is_found_with_its_length() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    assert_eq!(files.found(&file).unwrap().length, 8);
}

#[test]
fn a_file_is_found_with_its_permission_bits() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    files.write(&file, b"written\n").unwrap();
    files.set_mode(&file, 0o640).unwrap();
    assert_eq!(files.found(&file).unwrap().mode, 0o640);
}

#[test]
fn a_directory_is_found_with_its_permission_bits() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    files.set_mode(&made, 0o750).unwrap();
    assert_eq!(files.found(&made).unwrap().mode, 0o750);
}

#[test]
fn a_symbolic_link_is_followed_to_the_permission_bits_of_what_it_leads_to() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let file = directory.path().join("file");
    let link = directory.path().join("link");
    files.write(&file, b"written\n").unwrap();
    files.link(Path::new("file"), &link).unwrap();
    files.set_mode(&file, 0o640).unwrap();
    assert_eq!(files.followed(&link).unwrap().mode, 0o640);
}

#[test]
fn a_path_is_resolved_through_its_symbolic_links() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    let link = directory.path().join("link");
    files.make_directory(&made).unwrap();
    files.write(&made.join("file"), b"written\n").unwrap();
    files.link(Path::new("made"), &link).unwrap();
    assert_eq!(
        files.resolved(&link.join("file")).unwrap(),
        files.resolved(&made).unwrap().join("file")
    );
}

#[test]
fn a_path_with_a_step_back_is_resolved_to_where_it_leads() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let made = directory.path().join("made");
    files.make_directory(&made).unwrap();
    assert_eq!(
        files.resolved(&made.join("..")).unwrap(),
        files.resolved(directory.path()).unwrap()
    );
}

#[test]
fn a_path_that_leads_nowhere_is_not_resolved() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let missing = directory.path().join("missing");
    assert_eq!(
        files.resolved(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_path_through_a_symbolic_link_that_leads_nowhere_is_not_resolved() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let link = directory.path().join("link");
    files.link(Path::new("missing"), &link).unwrap();
    assert_eq!(
        files.resolved(&link).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn a_path_through_symbolic_links_that_lead_to_each_other_is_not_resolved() {
    let directory = test_inputs::temporary_directory();
    let files = Disk;
    let link = directory.path().join("link");
    files.link(Path::new("other"), &link).unwrap();
    files
        .link(Path::new("link"), &directory.path().join("other"))
        .unwrap();
    assert!(files.resolved(&link).is_err());
}

#[test]
fn a_device_is_found_as_neither_a_file_a_directory_nor_a_symbolic_link() {
    assert_eq!(
        Disk.found(Path::new("/dev/null")).unwrap().kind,
        Kind::Other
    );
}

#[test]
fn a_socket_is_found_as_neither_a_file_a_directory_nor_a_symbolic_link() {
    let directory = test_inputs::temporary_directory();
    let socket = directory.path().join("socket");
    let _listener = UnixListener::bind(&socket).unwrap();
    assert_eq!(Disk.found(&socket).unwrap().kind, Kind::Other);
}
