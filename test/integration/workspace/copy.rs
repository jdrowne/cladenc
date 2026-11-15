use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use cladenc::workspace::{copy_tree, holds_only_harness_files};
use tempfile::TempDir;

fn copied() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let copy = directory.path().join("copy");
    copy_tree(&progenitor, &copy).unwrap();
    (directory, copy)
}

fn with_a_socket(directory: &Path) -> PathBuf {
    let progenitor = test_inputs::progenitor(directory);
    let _socket = UnixListener::bind(progenitor.join("prompts/socket")).unwrap();
    progenitor
}

#[test]
fn a_copy_keeps_the_contents_of_a_file() {
    let (_directory, copy) = copied();
    assert_eq!(
        fs::read_to_string(copy.join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_copy_keeps_the_contents_of_a_file_in_a_directory() {
    let (_directory, copy) = copied();
    assert_eq!(
        fs::read_to_string(copy.join("prompts/system.txt")).unwrap(),
        "Solve the task.\n"
    );
}

#[test]
fn a_copy_keeps_the_permission_bits_of_a_file() {
    let (_directory, copy) = copied();
    let mode = fs::metadata(copy.join("harness.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o750);
}

#[test]
fn a_copy_keeps_a_symbolic_link_as_a_link() {
    let (_directory, copy) = copied();
    assert_eq!(
        fs::read_link(copy.join("prompt.txt")).unwrap(),
        Path::new("prompts/system.txt")
    );
}

#[test]
fn copying_a_tree_that_holds_a_socket_is_an_error() {
    let directory = test_inputs::temporary_directory();
    let tree = with_a_socket(directory.path());
    let refused = copy_tree(&tree, &directory.path().join("copy")).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::InvalidInput);
}

#[test]
fn copying_a_tree_that_does_not_exist_is_an_error() {
    let directory = test_inputs::temporary_directory();
    let refused = copy_tree(
        &directory.path().join("missing"),
        &directory.path().join("copy"),
    )
    .unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}

#[test]
fn a_tree_of_files_directories_and_symbolic_links_holds_only_what_a_harness_may() {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    assert!(holds_only_harness_files(&progenitor).unwrap());
}

#[test]
fn a_tree_that_holds_a_socket_holds_more_than_a_harness_may() {
    let directory = test_inputs::temporary_directory();
    let tree = with_a_socket(directory.path());
    assert!(!holds_only_harness_files(&tree).unwrap());
}

#[test]
fn a_tree_that_does_not_exist_cannot_be_looked_through() {
    let directory = test_inputs::temporary_directory();
    let refused = holds_only_harness_files(&directory.path().join("missing")).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}
