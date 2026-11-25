use std::process::Command;

use test_inputs::{one_after_another, printed_by};

const COMMANDS: [&str; 5] = [
    "echo first",
    "false",
    "printf '[%s]' unended",
    "true",
    "echo last",
];

fn printed() -> String {
    let ran = Command::new("sh")
        .args(["-c", &one_after_another(&COMMANDS)])
        .output()
        .unwrap();
    String::from_utf8(ran.stdout).unwrap()
}

#[test]
fn each_command_is_run_after_a_mark_is_printed() {
    assert_eq!(
        one_after_another(&["pwd", "id -u"]),
        "printf '\\036'\npwd\nprintf '\\036'\nid -u\n"
    );
}

#[test]
fn what_the_first_command_printed_is_told_apart_from_what_the_others_printed() {
    assert_eq!(printed_by("echo first", &COMMANDS, &printed()), "first\n");
}

#[test]
fn a_command_that_fails_does_not_end_the_commands_after_it() {
    assert_eq!(printed_by("echo last", &COMMANDS, &printed()), "last\n");
}

#[test]
fn what_a_command_printed_without_an_end_of_line_is_given_as_it_was_printed() {
    assert_eq!(
        printed_by("printf '[%s]' unended", &COMMANDS, &printed()),
        "[unended]"
    );
}

#[test]
fn a_command_that_printed_nothing_is_given_as_nothing() {
    assert_eq!(printed_by("true", &COMMANDS, &printed()), "");
}

#[test]
fn what_was_printed_before_the_first_mark_is_of_no_command() {
    assert_eq!(printed_by("pwd", &["pwd"], "before\u{1e}/app\n"), "/app\n");
}
