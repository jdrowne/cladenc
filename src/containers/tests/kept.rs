use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::benchmark::Artifact;
use crate::containers::artifacts;

fn place_of(destination: &str) -> PathBuf {
    let artifact = Artifact {
        source: "/app/reports".to_owned(),
        destination: destination.to_owned(),
        exclude: Vec::new(),
    };
    artifacts::place_of(
        &artifact,
        Path::new("/outputs/running/attempt-3-t1/artifacts"),
    )
}

fn kept_with(directories: &[&str], files: &[&str]) -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let kept = directory.path().join("artifacts");
    for made in directories {
        fs::create_dir_all(kept.join(made)).unwrap();
    }
    for file in files {
        fs::write(kept.join(file), "").unwrap();
    }
    (directory, kept)
}

fn to_be_given() -> (TempDir, PathBuf) {
    let directory = test_inputs::temporary_directory();
    let given = directory.path().join("given-to-the-grader");
    (directory, given)
}

#[test]
fn an_artifact_is_kept_at_its_destination_under_the_directory_of_the_artifacts() {
    assert_eq!(
        place_of("app/reports"),
        Path::new("/outputs/running/attempt-3-t1/artifacts/app/reports")
    );
}

#[test]
fn a_destination_written_with_slashes_to_spare_is_kept_at_the_same_place() {
    assert_eq!(place_of("/app//reports/"), place_of("app/reports"));
}

#[test]
fn the_empty_directories_above_a_place_are_removed_with_the_directory_of_the_artifacts() {
    let (_directory, kept) = kept_with(&["app/reports"], &[]);
    artifacts::remove_above_where_empty(&kept.join("app/reports/latest"), &kept);
    assert!(!kept.exists());
}

#[test]
fn a_directory_above_a_place_that_holds_something_stays_when_the_empty_one_under_it_goes() {
    let (_directory, kept) = kept_with(&["app/reports"], &["app/notes.txt"]);
    artifacts::remove_above_where_empty(&kept.join("app/reports/latest"), &kept);
    assert_eq!(
        (kept.join("app/reports").exists(), kept.join("app").exists()),
        (false, true)
    );
}

#[test]
fn no_directory_above_the_directory_of_the_artifacts_is_removed_however_empty() {
    let (directory, kept) = kept_with(&["app"], &[]);
    artifacts::remove_above_where_empty(&kept.join("app/report"), &kept);
    assert!(directory.path().exists());
}

#[test]
fn a_file_made_to_be_given_to_the_grader_is_removed() {
    let (_directory, given) = to_be_given();
    fs::write(&given, "").unwrap();
    artifacts::remove(&given);
    assert!(!given.exists());
}

#[test]
fn a_directory_made_to_be_given_to_the_grader_is_removed_with_all_it_holds() {
    let (_directory, given) = to_be_given();
    fs::create_dir_all(given.join("app")).unwrap();
    fs::write(given.join("app/report"), "").unwrap();
    artifacts::remove(&given);
    assert!(!given.exists());
}
