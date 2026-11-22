use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use crate::outputs_directory::copy::make_removable;
use tempfile::TempDir;

fn tree() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let tree = directory.path().join("tree");
    fs::create_dir_all(tree.join("inside")).unwrap();
    fs::write(tree.join("inside/file"), "kept\n").unwrap();
    (directory, tree)
}

fn with_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn mode_of(path: &Path) -> u32 {
    fs::symlink_metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn a_directory_made_removable_can_be_read_written_into_and_entered_by_its_owner() {
    let (_directory, tree) = tree();
    with_mode(&tree, 0o000);
    make_removable(&tree);
    assert_eq!(mode_of(&tree), 0o700);
}

#[test]
fn a_directory_made_removable_keeps_the_permission_bits_it_had_for_others() {
    let (_directory, tree) = tree();
    with_mode(&tree, 0o055);
    make_removable(&tree);
    assert_eq!(mode_of(&tree), 0o755);
}

#[test]
fn a_directory_inside_a_tree_made_removable_is_made_removable_too() {
    let (_directory, tree) = tree();
    with_mode(&tree.join("inside"), 0o000);
    make_removable(&tree);
    assert_eq!(mode_of(&tree.join("inside")), 0o700);
}

#[test]
fn a_file_in_a_tree_made_removable_keeps_its_permission_bits() {
    let (_directory, tree) = tree();
    with_mode(&tree.join("inside/file"), 0o400);
    make_removable(&tree);
    assert_eq!(mode_of(&tree.join("inside/file")), 0o400);
}

#[test]
fn a_file_made_removable_keeps_its_permission_bits() {
    let (_directory, tree) = tree();
    with_mode(&tree.join("inside/file"), 0o400);
    make_removable(&tree.join("inside/file"));
    assert_eq!(mode_of(&tree.join("inside/file")), 0o400);
}

#[test]
fn a_tree_whose_directories_could_not_be_written_into_can_be_removed_once_made_removable() {
    let (_directory, tree) = tree();
    with_mode(&tree.join("inside"), 0o500);
    with_mode(&tree, 0o500);
    make_removable(&tree);
    assert!(fs::remove_dir_all(&tree).is_ok());
}

#[test]
fn making_a_tree_removable_changes_nothing_a_symbolic_link_in_it_leads_to() {
    let (directory, tree) = tree();
    let outside = directory.path().join("outside");
    fs::create_dir(&outside).unwrap();
    with_mode(&outside, 0o500);
    symlink(&outside, tree.join("link")).unwrap();
    make_removable(&tree);
    assert_eq!(mode_of(&outside), 0o500);
}

#[test]
fn making_removable_a_tree_that_is_not_there_makes_nothing() {
    let directory = test_inputs::temporary_directory();
    let missing = directory.path().join("missing");
    make_removable(&missing);
    assert!(!missing.exists());
}
