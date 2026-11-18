use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use test_inputs::{primogenitor, temporary_directory};

fn written_in(file: &str) -> String {
    let directory = temporary_directory();
    fs::read_to_string(primogenitor(directory.path()).join(file)).unwrap()
}

#[test]
fn a_primogenitor_is_the_directory_named_primogenitor() {
    let directory = temporary_directory();
    assert_eq!(
        primogenitor(directory.path()),
        directory.path().join("primogenitor")
    );
}

#[test]
fn a_primogenitor_holds_a_script_a_directory_of_prompts_and_a_link_and_no_more() {
    let directory = temporary_directory();
    let mut held: Vec<String> = fs::read_dir(primogenitor(directory.path()))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    held.sort();
    assert_eq!(held, ["harness.sh", "prompt.txt", "prompts"]);
}

#[test]
fn the_script_of_a_primogenitor_says_harness() {
    assert_eq!(written_in("harness.sh"), "#!/bin/sh\necho harness\n");
}

#[test]
fn the_script_of_a_primogenitor_can_be_run_by_its_owner_and_its_group_and_no_one_else() {
    let directory = temporary_directory();
    let script = primogenitor(directory.path()).join("harness.sh");
    assert_eq!(
        fs::metadata(script).unwrap().permissions().mode() & 0o777,
        0o750
    );
}

#[test]
fn the_prompt_of_a_primogenitor_is_among_its_prompts() {
    assert_eq!(written_in("prompts/system.txt"), "Solve the task.\n");
}

#[test]
fn the_link_of_a_primogenitor_is_a_symbolic_link_to_its_prompt() {
    let directory = temporary_directory();
    let link = primogenitor(directory.path()).join("prompt.txt");
    assert_eq!(
        fs::read_link(link).unwrap(),
        PathBuf::from("prompts/system.txt")
    );
}

#[test]
fn the_prompt_of_a_primogenitor_is_read_through_its_link() {
    assert_eq!(written_in("prompt.txt"), "Solve the task.\n");
}
