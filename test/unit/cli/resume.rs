use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli, Command, ResumeOptions};
use clap::Parser;
use clap::error::ErrorKind;

struct Ran {
    status: u8,
    output: String,
    errors: String,
}

fn parsed(options: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(["cladenc", "resume"].iter().chain(options).copied())
}

fn ran(run_dir: PathBuf, quiet: bool) -> Ran {
    let cli = Cli {
        command: Command::Resume(ResumeOptions { run_dir, quiet }),
    };
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let status = cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut errors);
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}

fn missing() -> PathBuf {
    test_inputs::temporary_directory().path().join("missing")
}

#[test]
fn the_run_directory_is_the_one_given() {
    assert!(matches!(
        parsed(&["--run-dir=/run"]).unwrap().command,
        Command::Resume(ResumeOptions { run_dir, .. }) if run_dir == Path::new("/run")
    ));
}

#[test]
fn leaving_out_the_run_directory_is_an_error() {
    assert_eq!(
        parsed(&[]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn quiet_takes_no_value() {
    assert!(parsed(&["--run-dir=/run", "--quiet=yes"]).is_err());
}

#[test]
fn writing_quiet_turns_it_on() {
    assert!(matches!(
        parsed(&["--run-dir=/run", "--quiet"]).unwrap().command,
        Command::Resume(ResumeOptions { quiet: true, .. })
    ));
}

#[test]
fn leaving_quiet_out_leaves_it_off() {
    assert!(matches!(
        parsed(&["--run-dir=/run"]).unwrap().command,
        Command::Resume(ResumeOptions { quiet: false, .. })
    ));
}

#[test]
fn a_run_directory_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(missing(), false).status, 2);
}

#[test]
fn a_run_directory_that_does_not_exist_is_reported_on_standard_error() {
    assert!(
        ran(missing(), false)
            .errors
            .starts_with("cladenc: the run directory cannot be used: ")
    );
}

#[test]
fn a_run_directory_that_does_not_exist_prints_nothing_on_standard_output() {
    assert_eq!(ran(missing(), false).output, "");
}

#[test]
fn with_quiet_a_run_directory_that_does_not_exist_prints_nothing_on_standard_error() {
    assert_eq!(ran(missing(), true).errors, "");
}

#[test]
fn a_run_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(PathBuf::new(), false).status, 2);
}
