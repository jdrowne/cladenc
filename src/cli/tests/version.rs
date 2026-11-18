use crate::cli::Cli;
use clap::Parser;
use clap::error::ErrorKind;

fn asked(option: &str) -> ErrorKind {
    Cli::try_parse_from(["cladenc", option]).unwrap_err().kind()
}

#[test]
fn the_long_option_asks_for_the_version_to_be_shown() {
    assert_eq!(asked("--version"), ErrorKind::DisplayVersion);
}

#[test]
fn the_short_option_asks_for_the_version_to_be_shown() {
    assert_eq!(asked("-V"), ErrorKind::DisplayVersion);
}
