use std::fs;

use test_inputs::{line_is_written, temporary_directory, within_ten_seconds};

fn written(text: &str) -> bool {
    let directory = temporary_directory();
    let file = directory.path().join("id");
    fs::write(&file, text).unwrap();
    line_is_written(&file)
}

#[test]
fn what_holds_at_the_first_look_holds_within_ten_seconds() {
    assert!(within_ten_seconds(|| true));
}

#[test]
fn what_holds_at_a_later_look_holds_within_ten_seconds() {
    let mut looks = 0;
    assert!(within_ten_seconds(|| {
        looks += 1;
        looks == 3
    }));
}

#[test]
fn what_holds_is_not_looked_at_again() {
    let mut looks = 0;
    within_ten_seconds(|| {
        looks += 1;
        looks >= 3
    });
    assert_eq!(looks, 3);
}

#[test]
fn what_never_holds_does_not_hold_within_ten_seconds() {
    assert!(!within_ten_seconds(|| false));
}

#[test]
fn a_file_that_ends_with_a_line_ending_holds_a_written_line() {
    assert!(written("an-id\n"));
}

#[test]
fn a_file_whose_line_has_no_ending_yet_holds_no_written_line() {
    assert!(!written("an-id"));
}

#[test]
fn an_empty_file_holds_no_written_line() {
    assert!(!written(""));
}

#[test]
fn a_file_that_is_not_there_holds_no_written_line() {
    let directory = temporary_directory();
    assert!(!line_is_written(&directory.path().join("id")));
}
