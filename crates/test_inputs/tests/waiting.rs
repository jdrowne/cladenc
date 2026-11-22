use std::fs::{self, File};
use std::path::Path;
use std::process::{Child, Command};

use test_inputs::{line_is_written, temporary_directory, wait_for_a_line, within_ten_seconds};

const WRITES_A_LINE_AND_STAYS: &str = "echo an-id > id; exec sleep 300";
const WRITES_A_LINE_AND_ENDS: &str = "echo an-id > id";
const ENDS_WITH_NO_LINE_WRITTEN: &str = "echo 'no such image' >&2; exit 2";

fn written(text: &str) -> bool {
    let directory = temporary_directory();
    let file = directory.path().join("id");
    fs::write(&file, text).unwrap();
    line_is_written(&file)
}

fn program(directory: &Path, script: &str) -> Child {
    Command::new("sh")
        .args(["-c", script])
        .current_dir(directory)
        .stderr(File::create(directory.join("printed")).unwrap())
        .spawn()
        .unwrap()
}

fn waited_for(script: &str) -> (bool, bool) {
    let directory = temporary_directory();
    let mut program = program(directory.path(), script);
    let file = directory.path().join("id");
    wait_for_a_line(&mut program, &file, &directory.path().join("printed"));
    let still_runs = program.try_wait().unwrap().is_none();
    let _ = program.kill();
    program.wait().unwrap();
    (line_is_written(&file), still_runs)
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

#[test]
fn the_wait_for_a_line_ends_once_the_program_has_written_the_line() {
    let (written, _still_runs) = waited_for(WRITES_A_LINE_AND_STAYS);
    assert!(written);
}

#[test]
fn the_wait_for_a_line_leaves_the_program_running() {
    let (_written, still_runs) = waited_for(WRITES_A_LINE_AND_STAYS);
    assert!(still_runs);
}

#[test]
fn the_line_of_a_program_that_has_ended_is_waited_for_as_well() {
    let (written, _still_runs) = waited_for(WRITES_A_LINE_AND_ENDS);
    assert!(written);
}

#[test]
#[should_panic(expected = "the program wrote no line before it ended with exit status: 2")]
fn a_program_that_ends_with_no_line_written_is_reported_with_how_it_ended() {
    waited_for(ENDS_WITH_NO_LINE_WRITTEN);
}

#[test]
#[should_panic(expected = "and printed:\nno such image\n")]
fn a_program_that_ends_with_no_line_written_is_reported_with_what_it_printed() {
    waited_for(ENDS_WITH_NO_LINE_WRITTEN);
}
