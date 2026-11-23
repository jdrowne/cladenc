use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use test_inputs::{Disk, Files, temporary_directory};

#[test]
fn a_directory_is_made_on_the_disk_with_those_above_it() {
    let directory = temporary_directory();
    let made = directory.path().join("above").join("made");
    Disk.make_directories(&made);
    assert!(made.is_dir());
}

#[test]
fn a_file_is_written_on_the_disk_with_its_text() {
    let directory = temporary_directory();
    let file = directory.path().join("file");
    Disk.write(&file, "written\n");
    assert_eq!(fs::read_to_string(file).unwrap(), "written\n");
}

#[test]
fn a_file_on_the_disk_is_given_its_permission_bits() {
    let directory = temporary_directory();
    let file = directory.path().join("file");
    Disk.write(&file, "written\n");
    Disk.set_mode(&file, 0o750);
    assert_eq!(
        fs::metadata(file).unwrap().permissions().mode() & 0o7777,
        0o750
    );
}

#[test]
fn a_symbolic_link_is_made_on_the_disk_to_its_target_as_written() {
    let directory = temporary_directory();
    let link = directory.path().join("link");
    Disk.link("a/target", &link);
    assert_eq!(fs::read_link(link).unwrap(), PathBuf::from("a/target"));
}

#[test]
fn the_readme_is_the_one_of_the_repository() {
    let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    assert_eq!(test_inputs::README, fs::read_to_string(readme).unwrap());
}

#[test]
fn the_description_of_the_package_is_the_one_of_the_repository() {
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../etc/nfpm/nfpm.yaml");
    assert_eq!(test_inputs::PACKAGE, fs::read_to_string(package).unwrap());
}
