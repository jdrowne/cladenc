use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::containers::artifacts;

const EVERY_BIT: u32 = 0o7777;

struct Copied {
    directory: TempDir,
    copy: PathBuf,
    kind: Option<bool>,
}

fn copied(make: impl FnOnce(&Path)) -> Copied {
    let directory = test_inputs::temporary_directory();
    let kept = directory.path().join("kept");
    make(&kept);
    let copy = directory.path().join("copy");
    let kind = artifacts::trusted(&kept, &copy);
    Copied {
        directory,
        copy,
        kind,
    }
}

fn a_directory(make: impl FnOnce(&Path)) -> Copied {
    copied(|kept| {
        fs::create_dir_all(kept.join("inner")).unwrap();
        fs::write(kept.join("report.txt"), "found\n").unwrap();
        fs::write(kept.join("inner/notes.txt"), "noted\n").unwrap();
        make(kept);
    })
}

fn a_file_with_the_bits(mode: u32) -> Copied {
    copied(|kept| {
        fs::write(kept, "#!/bin/sh\n").unwrap();
        fs::set_permissions(kept, fs::Permissions::from_mode(mode)).unwrap();
    })
}

fn bits_of(copied: &Copied) -> u32 {
    fs::symlink_metadata(&copied.copy)
        .unwrap()
        .permissions()
        .mode()
        & EVERY_BIT
}

fn with_a_link(link: &str, target: &str) -> Option<PathBuf> {
    let copied = a_directory(|kept| symlink(target, kept.join(link)).unwrap());
    fs::read_link(copied.copy.join(link)).ok()
}

#[test]
fn a_path_that_was_not_kept_gives_nothing_to_hand_over() {
    assert_eq!(copied(|_| ()).kind, None);
}

#[test]
fn a_path_that_was_not_kept_leaves_no_copy() {
    assert!(!copied(|_| ()).copy.exists());
}

#[test]
fn a_file_that_was_kept_is_said_to_be_a_file() {
    assert_eq!(
        copied(|kept| fs::write(kept, "found\n").unwrap()).kind,
        Some(false)
    );
}

#[test]
fn a_file_that_was_kept_is_copied_with_what_it_holds() {
    let copied = copied(|kept| fs::write(kept, "found\n").unwrap());
    assert_eq!(fs::read_to_string(&copied.copy).unwrap(), "found\n");
}

#[test]
fn a_directory_that_was_kept_is_said_to_be_a_directory() {
    assert_eq!(a_directory(|_| ()).kind, Some(true));
}

#[test]
fn a_directory_that_was_kept_is_copied_with_the_files_it_holds() {
    let copied = a_directory(|_| ());
    assert_eq!(
        fs::read_to_string(copied.copy.join("report.txt")).unwrap(),
        "found\n"
    );
}

#[test]
fn a_directory_that_was_kept_is_copied_with_the_directories_inside_it() {
    let copied = a_directory(|_| ());
    assert_eq!(
        fs::read_to_string(copied.copy.join("inner/notes.txt")).unwrap(),
        "noted\n"
    );
}

#[test]
fn the_copy_leaves_what_was_kept_where_it_is() {
    let copied = a_directory(|_| ());
    assert!(
        copied
            .directory
            .path()
            .join("kept/inner/notes.txt")
            .is_file()
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
    let copied = a_directory(|kept| {
        fs::set_permissions(
            kept.join("inner/notes.txt"),
            fs::Permissions::from_mode(0o4755),
        )
        .unwrap();
    });
    assert_eq!(
        fs::symlink_metadata(copied.copy.join("inner/notes.txt"))
            .unwrap()
            .permissions()
            .mode()
            & EVERY_BIT,
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
    let directory = test_inputs::temporary_directory();
    let kept = directory.path().join("kept");
    fs::create_dir(&kept).unwrap();
    let _socket = UnixListener::bind(kept.join("socket")).unwrap();
    let copy = directory.path().join("copy");
    artifacts::trusted(&kept, &copy);
    assert_eq!(fs::read_dir(&copy).unwrap().count(), 0);
}

#[test]
fn a_path_that_was_kept_as_a_symbolic_link_gives_nothing_to_hand_over() {
    assert_eq!(
        copied(|kept| symlink("/tests/solution.txt", kept).unwrap()).kind,
        None
    );
}

#[test]
fn a_path_that_was_kept_as_a_socket_gives_nothing_to_hand_over() {
    let directory = test_inputs::temporary_directory();
    let kept = directory.path().join("kept");
    let _socket = UnixListener::bind(&kept).unwrap();
    assert_eq!(
        artifacts::trusted(&kept, &directory.path().join("copy")),
        None
    );
}
