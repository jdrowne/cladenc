use std::path::Path;

use crate::cli::{Cli, Command};
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 14] = [
    ("--primogenitor", "/primogenitor"),
    ("--model", "a-model"),
    ("--model-url", "http://models.example/v1"),
    ("--benchmark", "/benchmark"),
    ("--docker-image", "image"),
    ("--harness-run-command", "run a harness"),
    ("--outputs-dir", "/outputs"),
    ("--max-procreations", "800"),
    ("--genealogy-growth", "0.6"),
    ("--max-concurrent-containers", "8"),
    ("--procreation-timeout", "3600"),
    ("--attempt-timeout", "3600"),
    ("--holdout-percentage", "60"),
    ("--holdout-corroboration-percentage", "25"),
];

fn with(changed: &str, given: &str) -> Result<Cli, clap::Error> {
    let options = OPTIONS.iter().map(|(option, value)| {
        let value = if *option == changed { given } else { value };
        format!("{option}={value}")
    });
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options),
    )
}

fn and(another: Option<String>) -> Result<Cli, clap::Error> {
    let options = OPTIONS
        .iter()
        .map(|(option, value)| format!("{option}={value}"));
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options)
            .chain(another),
    )
}

fn built_by(command: Option<&str>) -> Result<Cli, clap::Error> {
    and(command.map(|command| format!("--harness-build-command={command}")))
}

fn with_the_key_in(file: Option<&str>) -> Result<Cli, clap::Error> {
    and(file.map(|file| format!("--model-key-file={file}")))
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
fn an_empty_model_is_an_error() {
    assert!(with("--model", "").is_err());
}

#[test]
fn an_empty_address_of_the_model_is_an_error() {
    assert!(with("--model-url", "").is_err());
}

#[test]
fn a_model_and_its_address_are_taken_as_they_are_written() {
    assert!(matches!(
        with("--model-url", "http://host.docker.internal:11434/v1").unwrap().command,
        Command::Start(options) if options.model == "a-model"
            && options.model_url == "http://host.docker.internal:11434/v1"
    ));
}

#[test]
fn a_command_line_with_no_model_key_file_names_no_file_of_a_key() {
    assert!(matches!(
        with_the_key_in(None).unwrap().command,
        Command::Start(options) if options.model_key_file.is_none()
    ));
}

#[test]
fn a_model_key_file_is_the_path_that_is_written() {
    assert!(matches!(
        with_the_key_in(Some("keys/model")).unwrap().command,
        Command::Start(options) if options.model_key_file.as_deref() == Some(Path::new("keys/model"))
    ));
}

#[test]
fn an_empty_model_key_file_is_an_error() {
    assert!(with_the_key_in(Some("")).is_err());
}

#[test]
fn a_benchmark_is_the_path_that_is_written() {
    assert!(matches!(
        with("--benchmark", "benchmarks/parsing").unwrap().command,
        Command::Start(options) if options.benchmark == [Path::new("benchmarks/parsing")]
    ));
}

#[test]
fn a_benchmark_given_a_second_time_is_a_second_benchmark() {
    assert!(matches!(
        and(Some("--benchmark=benchmarks/refactoring".to_owned())).unwrap().command,
        Command::Start(options)
            if options.benchmark == [Path::new("/benchmark"), Path::new("benchmarks/refactoring")]
    ));
}

#[test]
fn a_second_path_after_one_benchmark_flag_is_an_error() {
    let given = [
        "cladenc",
        "start",
        "--benchmark",
        "/benchmark",
        "/refactoring",
    ];
    assert_eq!(
        Cli::try_parse_from(given).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn an_empty_benchmark_is_an_error() {
    assert!(with("--benchmark", "").is_err());
}

#[test]
fn the_most_procreations_being_0_is_an_error() {
    assert_eq!(
        refusal("--max-procreations", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_procreations_being_1_is_accepted() {
    assert!(matches!(
        with("--max-procreations", "1").unwrap().command,
        Command::Start(options) if options.max_procreations == 1
    ));
}

#[test]
fn the_most_procreations_being_below_0_is_an_error() {
    assert_eq!(
        refusal("--max-procreations", "-1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_procreations_not_being_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-procreations", "many"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_budget_is_an_argument_that_is_not_expected() {
    assert_eq!(
        and(Some("--budget=800".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
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
fn a_confidence_is_an_argument_that_is_not_expected() {
    assert_eq!(
        and(Some("--confidence=99".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_limit_of_0_concurrent_containers_is_an_error() {
    assert_eq!(
        refusal("--max-concurrent-containers", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_limit_of_concurrent_containers_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-concurrent-containers", "many"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_limit_of_concurrent_containers_is_the_number_given() {
    assert!(matches!(
        with("--max-concurrent-containers", "3").unwrap().command,
        Command::Start(options) if options.max_concurrent_containers == 3
    ));
}

#[test]
fn the_limit_of_concurrent_containers_has_no_second_name() {
    assert_eq!(
        and(Some("--workers=8".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_procreation_timeout_of_0_is_an_error() {
    assert_eq!(
        refusal("--procreation-timeout", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn an_attempt_timeout_of_0_is_an_error() {
    assert_eq!(
        refusal("--attempt-timeout", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_seed_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--seed=7".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn initial_procreations_are_no_flag_of_start() {
    assert_eq!(
        and(Some("--initial-procreations=5".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn quiet_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--quiet".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_holdout_percentage_of_0_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_of_100_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "100"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_below_0_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "-5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_above_100_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "100.5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "NaN"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_written_with_a_percent_sign_is_an_error() {
    assert_eq!(
        refusal("--holdout-percentage", "60%"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_percentage_that_is_refused_is_said_to_take_a_number_above_0_and_below_100() {
    assert!(
        with("--holdout-percentage", "100")
            .unwrap_err()
            .to_string()
            .contains("a number above 0 and below 100 is expected")
    );
}

#[test]
fn a_holdout_percentage_is_the_number_given() {
    assert!(matches!(
        with("--holdout-percentage", "60").unwrap().command,
        Command::Start(options) if options.holdout_percentage == 60.0
    ));
}

#[test]
fn a_holdout_percentage_with_a_decimal_point_is_the_number_given() {
    assert!(matches!(
        with("--holdout-percentage", "12.5").unwrap().command,
        Command::Start(options) if options.holdout_percentage == 12.5
    ));
}

#[test]
fn a_holdout_percentage_just_above_0_is_accepted() {
    assert!(with("--holdout-percentage", "0.1").is_ok());
}

#[test]
fn a_holdout_percentage_just_below_100_is_accepted() {
    assert!(with("--holdout-percentage", "99.9").is_ok());
}

#[test]
fn a_part_that_is_held_out_has_no_flag_under_another_name() {
    assert_eq!(
        and(Some("--holdout=60".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_holdout_corroboration_percentage_of_0_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_of_100_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "100"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_below_0_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "-5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_above_100_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "100.5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "NaN"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_written_with_a_percent_sign_is_an_error() {
    assert_eq!(
        refusal("--holdout-corroboration-percentage", "25%"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_holdout_corroboration_percentage_that_is_refused_is_said_to_take_a_number_above_0_and_below_100()
 {
    assert!(
        with("--holdout-corroboration-percentage", "100")
            .unwrap_err()
            .to_string()
            .contains("a number above 0 and below 100 is expected")
    );
}

#[test]
fn a_holdout_corroboration_percentage_is_the_number_given() {
    assert!(matches!(
        with("--holdout-corroboration-percentage", "25").unwrap().command,
        Command::Start(options) if options.holdout_corroboration_percentage == 25.0
    ));
}

#[test]
fn a_holdout_corroboration_percentage_with_a_decimal_point_is_the_number_given() {
    assert!(matches!(
        with("--holdout-corroboration-percentage", "12.5").unwrap().command,
        Command::Start(options) if options.holdout_corroboration_percentage == 12.5
    ));
}

#[test]
fn a_holdout_corroboration_percentage_just_above_0_is_accepted() {
    assert!(with("--holdout-corroboration-percentage", "0.1").is_ok());
}

#[test]
fn a_holdout_corroboration_percentage_just_below_100_is_accepted() {
    assert!(with("--holdout-corroboration-percentage", "99.9").is_ok());
}

#[test]
fn the_part_of_the_holdout_tasks_that_is_only_scored_has_no_flag_under_another_name() {
    assert_eq!(
        and(Some("--corroboration-percentage=25".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}
