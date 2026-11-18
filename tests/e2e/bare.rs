use std::process::{Command, Output};

fn bare() -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .output()
        .unwrap()
}

#[test]
fn cladenc_with_no_sub_command_prints_its_sub_commands_on_standard_error() {
    assert!(
        String::from_utf8(bare().stderr)
            .unwrap()
            .contains("\nUsage: cladenc <COMMAND>\n\nCommands:\n  start ")
    );
}

#[test]
fn cladenc_with_no_sub_command_prints_nothing_on_standard_output() {
    assert_eq!(String::from_utf8(bare().stdout).unwrap(), "");
}

#[test]
fn cladenc_with_no_sub_command_exits_with_2() {
    assert_eq!(bare().status.code(), Some(2));
}
