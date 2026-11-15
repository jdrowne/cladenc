use cladenc::cli::{Cli, Command};
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 14] = [
    ("--progenitor", "/progenitor"),
    ("--tasks", "/tasks"),
    ("--docker-image", "image"),
    ("--harness-run-command", "run a harness"),
    ("--evaluator", "/evaluator"),
    ("--run-dir", "/run"),
    ("--budget", "800"),
    ("--genealogy-growth", "0.6"),
    ("--confidence", "99"),
    ("--initial-procreations", "5"),
    ("--workers", "8"),
    ("--procreation-timeout", "3600"),
    ("--evaluation-timeout", "3600"),
    ("--seed", "7"),
];

fn with(changed: &str, given: &str) -> Result<Cli, clap::Error> {
    let options = OPTIONS.iter().map(|(option, value)| {
        let value = if *option == changed { given } else { value };
        format!("{option}={value}")
    });
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options)
            .chain((changed == "--quiet").then_some(given.to_owned())),
    )
}

fn built_by(command: Option<&str>) -> Result<Cli, clap::Error> {
    let options = OPTIONS
        .iter()
        .map(|(option, value)| format!("{option}={value}"));
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options)
            .chain(command.map(|command| format!("--harness-build-command={command}"))),
    )
}

fn refusal(changed: &str, given: &str) -> ErrorKind {
    with(changed, given).unwrap_err().kind()
}

#[test]
fn an_empty_docker_image_is_an_error() {
    assert!(with("--docker-image", "").is_err());
}

#[test]
fn an_empty_harness_run_command_is_an_error() {
    assert!(with("--harness-run-command", "").is_err());
}

#[test]
fn a_harness_run_command_is_taken_as_it_is_written() {
    assert!(matches!(
        with("--harness-run-command", "sh \"$CLADENC_HARNESS/harness.sh\" --flag").unwrap().command,
        Command::Start(options)
            if options.harness_run_command == "sh \"$CLADENC_HARNESS/harness.sh\" --flag"
    ));
}

#[test]
fn a_command_line_with_no_harness_build_command_builds_nothing() {
    assert!(matches!(
        built_by(None).unwrap().command,
        Command::Start(options) if options.harness_build_command.is_none()
    ));
}

#[test]
fn a_harness_build_command_is_taken_as_it_is_written() {
    assert!(matches!(
        built_by(Some("cargo build --release && cp target/release/harness .")).unwrap().command,
        Command::Start(options) if options.harness_build_command.as_deref()
            == Some("cargo build --release && cp target/release/harness .")
    ));
}

#[test]
fn an_empty_harness_build_command_is_an_error() {
    assert!(built_by(Some("")).is_err());
}

#[test]
fn a_budget_of_0_is_an_error() {
    assert_eq!(refusal("--budget", "0"), ErrorKind::ValueValidation);
}

#[test]
fn a_budget_below_0_is_an_error() {
    assert_eq!(refusal("--budget", "-1"), ErrorKind::ValueValidation);
}

#[test]
fn a_budget_that_is_not_a_number_is_an_error() {
    assert_eq!(refusal("--budget", "many"), ErrorKind::ValueValidation);
}

#[test]
fn a_genealogy_growth_below_0_is_an_error() {
    assert_eq!(
        refusal("--genealogy-growth", "-0.1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_genealogy_growth_above_1_is_an_error() {
    assert_eq!(
        refusal("--genealogy-growth", "1.1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_genealogy_growth_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--genealogy-growth", "NaN"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_genealogy_growth_of_0_is_accepted() {
    assert!(with("--genealogy-growth", "0").is_ok());
}

#[test]
fn a_genealogy_growth_of_1_is_accepted() {
    assert!(with("--genealogy-growth", "1").is_ok());
}

#[test]
fn a_confidence_of_0_is_an_error() {
    assert_eq!(refusal("--confidence", "0"), ErrorKind::ValueValidation);
}

#[test]
fn a_confidence_of_100_is_an_error() {
    assert_eq!(refusal("--confidence", "100"), ErrorKind::ValueValidation);
}

#[test]
fn a_confidence_that_is_not_a_number_is_an_error() {
    assert_eq!(refusal("--confidence", "sure"), ErrorKind::ValueValidation);
}

#[test]
fn a_confidence_between_0_and_100_is_accepted() {
    assert!(with("--confidence", "99.9").is_ok());
}

#[test]
fn initial_procreations_below_0_are_an_error() {
    assert_eq!(
        refusal("--initial-procreations", "-1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn no_initial_procreations_are_accepted() {
    assert!(with("--initial-procreations", "0").is_ok());
}

#[test]
fn no_workers_are_an_error() {
    assert_eq!(refusal("--workers", "0"), ErrorKind::ValueValidation);
}

#[test]
fn a_procreation_timeout_of_0_is_an_error() {
    assert_eq!(
        refusal("--procreation-timeout", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn an_evaluation_timeout_of_0_is_an_error() {
    assert_eq!(
        refusal("--evaluation-timeout", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_seed_below_0_is_an_error() {
    assert_eq!(refusal("--seed", "-1"), ErrorKind::ValueValidation);
}

#[test]
fn a_seed_above_the_largest_is_an_error() {
    assert_eq!(
        refusal("--seed", "18446744073709551616"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_largest_seed_is_accepted() {
    assert!(with("--seed", "18446744073709551615").is_ok());
}

#[test]
fn quiet_takes_no_value() {
    assert!(with("--quiet", "--quiet=yes").is_err());
}

#[test]
fn writing_quiet_turns_it_on() {
    assert!(matches!(
        with("--quiet", "--quiet").unwrap().command,
        Command::Start(options) if options.quiet
    ));
}

#[test]
fn leaving_quiet_out_leaves_it_off() {
    assert!(matches!(
        with("", "").unwrap().command,
        Command::Start(options) if !options.quiet
    ));
}
