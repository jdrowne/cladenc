use std::process::{Command, Output};
use std::sync::LazyLock;

static BARE: LazyLock<Output> = LazyLock::new(|| {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .output()
        .unwrap()
});

#[test]
fn cladenc_with_no_sub_command_prints_its_sub_commands_on_standard_error() {
    assert!(
        str::from_utf8(&BARE.stderr)
            .unwrap()
            .contains("\nUsage: cladenc <COMMAND>\n\nCommands:\n  start ")
    );
}

#[test]
fn cladenc_with_no_sub_command_prints_nothing_on_standard_output() {
    assert_eq!(str::from_utf8(&BARE.stdout).unwrap(), "");
}

#[test]
fn cladenc_with_no_sub_command_exits_with_2() {
    assert_eq!(BARE.status.code(), Some(2));
}
