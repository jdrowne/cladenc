use std::process::Command;

use cladenc::containers::{Docker, Program};

fn sh(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

#[test]
fn what_a_command_that_succeeds_printed_is_read_as_it_was_printed() {
    let listed = Program.print(sh("echo cladenc-0123; echo cladenc-4567"));
    assert_eq!(listed.unwrap(), "cladenc-0123\ncladenc-4567\n");
}

#[test]
fn a_command_that_fails_is_an_error_of_what_it_said_went_wrong_and_not_of_what_it_printed() {
    let failed = Program.print(sh("echo listed; echo ' no such container ' >&2; exit 1"));
    assert_eq!(failed.unwrap_err().to_string(), "no such container");
}

#[test]
fn a_command_whose_program_cannot_be_started_is_an_error() {
    let missing = Program.print(Command::new("/no/such/program"));
    assert!(missing.is_err());
}

#[test]
fn a_command_is_given_nothing_on_its_standard_input() {
    let read = Program.print(sh("cat; echo end"));
    assert_eq!(read.unwrap(), "end\n");
}
