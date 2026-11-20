use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use cladenc::outputs_directory::{copy_tree, holds_only_harness_files, same_tree};
use tempfile::TempDir;

fn copied() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let copy = directory.path().join("copy");
    copy_tree(&primogenitor, &copy).unwrap();
    (directory, copy)
}

fn the_same_after(change: impl FnOnce(&Path)) -> bool {
    let (directory, copy) = copied();
    change(&copy);
    same_tree(&copy, &directory.path().join("primogenitor")).unwrap()
}

fn with_a_socket(directory: &Path) -> PathBuf {
    let primogenitor = test_inputs::primogenitor(directory);
    let _socket = UnixListener::bind(primogenitor.join("prompts/socket")).unwrap();
    primogenitor
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
    let primogenitor = test_inputs::primogenitor(directory.path());
    assert!(holds_only_harness_files(&primogenitor).unwrap());
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

#[test]
fn a_copy_of_a_tree_is_the_same_as_the_tree() {
    assert!(the_same_after(|_| {}));
}

#[test]
fn a_copy_whose_file_was_written_again_with_the_contents_it_had_is_the_same_as_the_tree() {
    assert!(the_same_after(|copy| {
        fs::write(copy.join("harness.sh"), "#!/bin/sh\necho harness\n").unwrap();
    }));
}

#[test]
fn a_copy_whose_file_holds_other_contents_of_the_same_length_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::write(copy.join("harness.sh"), "#!/bin/sh\necho HARNESS\n").unwrap();
    }));
}

#[test]
fn a_copy_whose_file_holds_more_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::write(
            copy.join("harness.sh"),
            "#!/bin/sh\necho harness\necho more\n",
        )
        .unwrap();
    }));
}

#[test]
fn a_copy_whose_file_in_a_directory_holds_other_contents_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::write(copy.join("prompts/system.txt"), "Solve it.\n").unwrap();
    }));
}

#[test]
fn a_copy_whose_file_has_other_permission_bits_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::set_permissions(copy.join("harness.sh"), fs::Permissions::from_mode(0o700)).unwrap();
    }));
}

#[test]
fn a_copy_whose_directory_has_other_permission_bits_is_the_same_as_the_tree() {
    assert!(the_same_after(|copy| {
        fs::set_permissions(copy.join("prompts"), fs::Permissions::from_mode(0o700)).unwrap();
    }));
}

#[test]
fn a_copy_that_holds_a_file_more_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::write(copy.join("notes.txt"), "").unwrap();
    }));
}

#[test]
fn a_copy_that_holds_a_file_fewer_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::remove_file(copy.join("prompts/system.txt")).unwrap();
    }));
}

#[test]
fn a_copy_that_holds_an_empty_directory_more_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::create_dir(copy.join("empty")).unwrap();
    }));
}

#[test]
fn a_copy_whose_symbolic_link_has_another_target_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::remove_file(copy.join("prompt.txt")).unwrap();
        symlink("harness.sh", copy.join("prompt.txt")).unwrap();
    }));
}

#[test]
fn a_copy_that_holds_a_file_in_the_place_of_a_symbolic_link_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::remove_file(copy.join("prompt.txt")).unwrap();
        fs::write(copy.join("prompt.txt"), "Solve the task.\n").unwrap();
    }));
}

#[test]
fn a_copy_that_holds_a_file_in_the_place_of_a_directory_is_not_the_same_as_the_tree() {
    assert!(!the_same_after(|copy| {
        fs::remove_dir_all(copy.join("prompts")).unwrap();
        fs::write(copy.join("prompts"), "").unwrap();
    }));
}

#[test]
fn a_tree_that_does_not_exist_cannot_be_compared_with_another() {
    let (directory, copy) = copied();
    let refused = same_tree(&directory.path().join("missing"), &copy).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}

#[test]
fn a_tree_cannot_be_compared_with_one_that_does_not_exist() {
    let (directory, copy) = copied();
    let refused = same_tree(&copy, &directory.path().join("missing")).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
}
