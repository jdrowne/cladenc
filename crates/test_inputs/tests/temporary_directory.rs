use std::fs;

use test_inputs::temporary_directory;

#[test]
fn a_temporary_directory_is_there() {
    assert!(temporary_directory().path().is_dir());
}

#[test]
fn a_temporary_directory_is_empty() {
    assert_eq!(
        fs::read_dir(temporary_directory().path()).unwrap().count(),
        0
    );
}

#[test]
fn two_temporary_directories_are_two() {
    assert_ne!(temporary_directory().path(), temporary_directory().path());
}

#[test]
fn a_temporary_directory_is_removed_once_it_is_let_go() {
    let directory = temporary_directory();
    let path = directory.path().to_owned();
    drop(directory);
    assert!(!path.exists());
}
