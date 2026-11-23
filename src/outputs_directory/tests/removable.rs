use std::path::{Path, PathBuf};

use crate::file_system::tests::double::Memory;
use crate::outputs_directory::copy::make_removable;

fn tree() -> (Memory, PathBuf) {
    let directory = Memory::default();
    let tree = directory.path().join("tree");
    directory.create_dir_all(tree.join("inside")).unwrap();
    directory.write(tree.join("inside/file"), "kept\n").unwrap();
    (directory, tree)
}

fn with_mode(directory: &Memory, path: &Path, mode: u32) {
    directory.set_mode(path, mode).unwrap();
}

fn mode_of(directory: &Memory, path: &Path) -> u32 {
    directory.mode(path) & 0o777
}

#[test]
fn a_directory_made_removable_can_be_read_written_into_and_entered_by_its_owner() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree, 0o000);
    make_removable(&directory, &tree);
    assert_eq!(mode_of(&directory, &tree), 0o700);
}

#[test]
fn a_directory_made_removable_keeps_the_permission_bits_it_had_for_others() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree, 0o055);
    make_removable(&directory, &tree);
    assert_eq!(mode_of(&directory, &tree), 0o755);
}

#[test]
fn a_directory_inside_a_tree_made_removable_is_made_removable_too() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree.join("inside"), 0o000);
    make_removable(&directory, &tree);
    assert_eq!(mode_of(&directory, &tree.join("inside")), 0o700);
}

#[test]
fn a_file_in_a_tree_made_removable_keeps_its_permission_bits() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree.join("inside/file"), 0o400);
    make_removable(&directory, &tree);
    assert_eq!(mode_of(&directory, &tree.join("inside/file")), 0o400);
}

#[test]
fn a_file_made_removable_keeps_its_permission_bits() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree.join("inside/file"), 0o400);
    make_removable(&directory, &tree.join("inside/file"));
    assert_eq!(mode_of(&directory, &tree.join("inside/file")), 0o400);
}

#[test]
fn a_tree_whose_directories_could_not_be_written_into_can_be_removed_once_made_removable() {
    let (directory, tree) = tree();
    with_mode(&directory, &tree.join("inside"), 0o500);
    with_mode(&directory, &tree, 0o500);
    make_removable(&directory, &tree);
    assert!(directory.remove_dir_all(&tree).is_ok());
}

#[test]
fn making_a_tree_removable_changes_nothing_a_symbolic_link_in_it_leads_to() {
    let (directory, tree) = tree();
    let outside = directory.path().join("outside");
    directory.create_dir(&outside).unwrap();
    with_mode(&directory, &outside, 0o500);
    directory.symlink(&outside, tree.join("link")).unwrap();
    make_removable(&directory, &tree);
    assert_eq!(mode_of(&directory, &outside), 0o500);
}

#[test]
fn making_removable_a_tree_that_is_not_there_makes_nothing() {
    let directory = Memory::default();
    let missing = directory.path().join("missing");
    make_removable(&directory, &missing);
    assert!(!directory.exists(&missing));
}
