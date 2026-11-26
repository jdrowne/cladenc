use std::path::{Path, PathBuf};

use crate::file_system::tests::double::Memory;
use crate::sandboxes::artifacts;

const EVERY_BIT: u32 = 0o7777;

struct Copied {
    directory: Memory,
    copy: PathBuf,
    kind: Option<bool>,
}

fn copied(make: impl FnOnce(&Memory, &Path)) -> Copied {
    let directory = Memory::default();
    let kept = directory.path().join("kept");
    make(&directory, &kept);
    let copy = directory.path().join("copy");
    let kind = artifacts::trusted(&directory, &kept, &copy);
    Copied {
        directory,
        copy,
        kind,
    }
}

fn a_directory(make: impl FnOnce(&Memory, &Path)) -> Copied {
    copied(|directory, kept| {
        directory.create_dir_all(kept.join("inner")).unwrap();
        directory.write(kept.join("report.txt"), "found\n").unwrap();
        directory
            .write(kept.join("inner/notes.txt"), "noted\n")
            .unwrap();
        make(directory, kept);
    })
}

fn a_file_with_the_bits(mode: u32) -> Copied {
    copied(|directory, kept| {
        directory.write(kept, "#!/bin/sh\n").unwrap();
        directory.set_mode(kept, mode).unwrap();
    })
}

fn bits_of(copied: &Copied) -> u32 {
    copied.directory.mode(&copied.copy) & EVERY_BIT
}

fn with_a_link(link: &str, target: &str) -> Option<PathBuf> {
    let copied = a_directory(|directory, kept| {
        directory.symlink(target, kept.join(link)).unwrap();
    });
    copied.directory.read_link(copied.copy.join(link)).ok()
}

#[test]
fn a_path_that_was_not_kept_gives_nothing_to_hand_over() {
    assert_eq!(copied(|_, _| ()).kind, None);
}

#[test]
fn a_path_that_was_not_kept_leaves_no_copy() {
    let copied = copied(|_, _| ());
    assert!(!copied.directory.exists(&copied.copy));
}

#[test]
fn a_file_that_was_kept_is_said_to_be_a_file() {
    assert_eq!(
        copied(|directory, kept| directory.write(kept, "found\n").unwrap()).kind,
        Some(false)
    );
}

#[test]
fn a_file_that_was_kept_is_copied_with_what_it_holds() {
    let copied = copied(|directory, kept| directory.write(kept, "found\n").unwrap());
    assert_eq!(
        copied.directory.read_to_string(&copied.copy).unwrap(),
        "found\n"
    );
}

#[test]
fn a_directory_that_was_kept_is_said_to_be_a_directory() {
    assert_eq!(a_directory(|_, _| ()).kind, Some(true));
}

#[test]
fn a_directory_that_was_kept_is_copied_with_the_files_it_holds() {
    let copied = a_directory(|_, _| ());
    assert_eq!(
        copied
            .directory
            .read_to_string(copied.copy.join("report.txt"))
            .unwrap(),
        "found\n"
    );
}

#[test]
fn a_directory_that_was_kept_is_copied_with_the_directories_inside_it() {
    let copied = a_directory(|_, _| ());
    assert_eq!(
        copied
            .directory
            .read_to_string(copied.copy.join("inner/notes.txt"))
            .unwrap(),
        "noted\n"
    );
}

#[test]
fn the_copy_leaves_what_was_kept_where_it_is() {
    let copied = a_directory(|_, _| ());
    assert!(
        copied
            .directory
            .is_file(copied.directory.path().join("kept/inner/notes.txt"))
    );
}

#[test]
fn a_file_keeps_the_bits_that_say_who_may_read_write_and_run_it() {
    assert_eq!(bits_of(&a_file_with_the_bits(0o750)), 0o750);
}

#[test]
fn a_file_does_not_keep_its_setuid_bit() {
    assert_eq!(bits_of(&a_file_with_the_bits(0o4755)), 0o755);
}

#[test]
fn a_file_does_not_keep_its_setgid_bit() {
    assert_eq!(bits_of(&a_file_with_the_bits(0o2755)), 0o755);
}

#[test]
fn a_file_inside_a_directory_does_not_keep_its_setuid_bit() {
    let copied = a_directory(|directory, kept| {
        directory
            .set_mode(kept.join("inner/notes.txt"), 0o4755)
            .unwrap();
    });
    assert_eq!(
        copied.directory.mode(copied.copy.join("inner/notes.txt")) & EVERY_BIT,
        0o755
    );
}

#[test]
fn a_symbolic_link_to_a_file_beside_it_is_kept() {
    assert_eq!(
        with_a_link("link", "report.txt"),
        Some(PathBuf::from("report.txt"))
    );
}

#[test]
fn a_symbolic_link_written_from_its_own_directory_is_kept() {
    assert_eq!(
        with_a_link("link", "./inner/notes.txt"),
        Some(PathBuf::from("./inner/notes.txt"))
    );
}

#[test]
fn a_symbolic_link_that_steps_back_and_stays_inside_the_path_is_kept() {
    assert_eq!(
        with_a_link("inner/link", "../report.txt"),
        Some(PathBuf::from("../report.txt"))
    );
}

#[test]
fn a_symbolic_link_to_nothing_inside_the_path_is_kept() {
    assert_eq!(
        with_a_link("link", "never/made"),
        Some(PathBuf::from("never/made"))
    );
}

#[test]
fn a_symbolic_link_that_steps_out_of_the_path_is_left_out() {
    assert_eq!(with_a_link("link", "../secret"), None);
}

#[test]
fn a_symbolic_link_inside_a_directory_that_steps_out_of_the_path_is_left_out() {
    assert_eq!(with_a_link("inner/link", "../../secret"), None);
}

#[test]
fn a_symbolic_link_that_steps_out_of_the_path_and_back_in_is_left_out() {
    assert_eq!(with_a_link("link", "inner/../../kept/report.txt"), None);
}

#[test]
fn a_symbolic_link_to_a_path_from_the_top_is_left_out() {
    assert_eq!(with_a_link("link", "/tests/solution.txt"), None);
}

#[test]
fn a_socket_inside_a_directory_is_left_out() {
    let directory = Memory::default();
    let kept = directory.path().join("kept");
    directory.create_dir(&kept).unwrap();
    directory.socket(kept.join("socket")).unwrap();
    let copy = directory.path().join("copy");
    artifacts::trusted(&directory, &kept, &copy);
    assert_eq!(directory.names_in(&copy).len(), 0);
}

#[test]
fn a_path_that_was_kept_as_a_symbolic_link_gives_nothing_to_hand_over() {
    assert_eq!(
        copied(|directory, kept| directory.symlink("/tests/solution.txt", kept).unwrap()).kind,
        None
    );
}

#[test]
fn a_path_that_was_kept_as_a_socket_gives_nothing_to_hand_over() {
    let directory = Memory::default();
    let kept = directory.path().join("kept");
    directory.socket(&kept).unwrap();
    assert_eq!(
        artifacts::trusted(&directory, &kept, &directory.path().join("copy")),
        None
    );
}
