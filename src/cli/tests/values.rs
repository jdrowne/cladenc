use std::path::Path;

use crate::cli::{Cli, Command};
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 14] = [
    ("--primogenitor", "/primogenitor"),
    ("--model", "a-model"),
    ("--model-url", "http://models.example/v1"),
    ("--benchmark", "/benchmark"),
    ("--diversification-docker-image", "image"),
    ("--harness-run-command", "run a harness"),
    ("--outputs-dir", "/outputs"),
    ("--max-diversifications", "800"),
    ("--max-concurrent-sandboxes", "8"),
    ("--diversification-timeout", "3600"),
    ("--max-verifier-health-check-retries", "3"),
    ("--max-task-health-check-retries", "2"),
    ("--selection-tasks-percentage", "45"),
    ("--corroboration-tasks-percentage", "15"),
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
    assert!(with("--diversification-docker-image", "").is_err());
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
fn the_most_diversifications_being_0_is_an_error() {
    assert_eq!(
        refusal("--max-diversifications", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_diversifications_being_1_is_accepted() {
    assert!(matches!(
        with("--max-diversifications", "1").unwrap().command,
        Command::Start(options) if options.max_diversifications == 1
    ));
}

#[test]
fn the_most_diversifications_being_below_0_is_an_error() {
    assert_eq!(
        refusal("--max-diversifications", "-1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_diversifications_not_being_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-diversifications", "many"),
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
fn a_genealogy_growth_is_an_argument_that_is_not_expected() {
    assert_eq!(
        and(Some("--genealogy-growth=0.6".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_confidence_is_an_argument_that_is_not_expected() {
    assert_eq!(
        and(Some("--confidence=99".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_limit_of_0_concurrent_sandboxes_is_an_error() {
    assert_eq!(
        refusal("--max-concurrent-sandboxes", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_limit_of_concurrent_sandboxes_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-concurrent-sandboxes", "many"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_limit_of_concurrent_sandboxes_is_the_number_given() {
    assert!(matches!(
        with("--max-concurrent-sandboxes", "3").unwrap().command,
        Command::Start(options) if options.max_concurrent_sandboxes == 3
    ));
}

#[test]
fn the_limit_of_concurrent_sandboxes_has_no_second_name() {
    assert_eq!(
        and(Some("--max-concurrent-containers=8".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_primogenitor_has_no_second_name() {
    assert_eq!(
        and(Some("--progenitor=/primogenitor".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_most_retries_of_the_health_check_of_a_verifier_are_the_number_given() {
    assert!(matches!(
        with("--max-verifier-health-check-retries", "5").unwrap().command,
        Command::Start(options) if options.max_verifier_health_check_retries == 5
    ));
}

#[test]
fn the_most_retries_of_the_health_check_of_a_verifier_being_0_is_accepted() {
    assert!(matches!(
        with("--max-verifier-health-check-retries", "0").unwrap().command,
        Command::Start(options) if options.max_verifier_health_check_retries == 0
    ));
}

#[test]
fn the_most_retries_of_the_health_check_of_a_verifier_being_below_0_is_an_error() {
    assert_eq!(
        refusal("--max-verifier-health-check-retries", "-1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_retries_of_the_health_check_of_a_verifier_not_being_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-verifier-health-check-retries", "many"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_retries_of_the_health_check_of_a_task_are_the_number_given() {
    assert!(matches!(
        with("--max-task-health-check-retries", "5").unwrap().command,
        Command::Start(options) if options.max_task_health_check_retries == 5
    ));
}

#[test]
fn the_most_retries_of_the_health_check_of_a_task_being_0_is_accepted() {
    assert!(matches!(
        with("--max-task-health-check-retries", "0").unwrap().command,
        Command::Start(options) if options.max_task_health_check_retries == 0
    ));
}

#[test]
fn the_most_retries_of_the_health_check_of_a_task_being_below_0_is_an_error() {
    assert_eq!(
        refusal("--max-task-health-check-retries", "-1"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_most_retries_of_the_health_check_of_a_task_not_being_a_number_is_an_error() {
    assert_eq!(
        refusal("--max-task-health-check-retries", "many"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_diversification_timeout_of_0_is_an_error() {
    assert_eq!(
        refusal("--diversification-timeout", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn the_older_name_of_the_flag_of_the_image_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--docker-image=image".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_older_name_of_the_flag_of_the_dockerfile_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--dockerfile=Dockerfile".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_older_name_of_the_flag_of_the_time_limit_of_the_build_of_the_image_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--docker-build-timeout=600".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_time_limit_of_a_trial_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--trial-timeout=3600".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
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
fn initial_diversifications_are_no_flag_of_start() {
    assert_eq!(
        and(Some("--initial-diversifications=5".to_owned()))
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
fn a_selection_tasks_percentage_of_0_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_of_100_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "100"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_below_0_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "-5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_above_100_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "100.5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "NaN"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_written_with_a_percent_sign_is_an_error() {
    assert_eq!(
        refusal("--selection-tasks-percentage", "45%"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_selection_tasks_percentage_that_is_refused_is_said_to_take_a_number_above_0_and_below_100() {
    assert!(
        with("--selection-tasks-percentage", "100")
            .unwrap_err()
            .to_string()
            .contains("a number above 0 and below 100 is expected")
    );
}

#[test]
fn a_selection_tasks_percentage_is_the_number_given() {
    assert!(matches!(
        with("--selection-tasks-percentage", "45").unwrap().command,
        Command::Start(options) if options.selection_tasks_percentage == 45.0
    ));
}

#[test]
fn a_selection_tasks_percentage_with_a_decimal_point_is_the_number_given() {
    assert!(matches!(
        with("--selection-tasks-percentage", "12.5").unwrap().command,
        Command::Start(options) if options.selection_tasks_percentage == 12.5
    ));
}

#[test]
fn a_selection_tasks_percentage_just_above_0_is_accepted() {
    assert!(with("--selection-tasks-percentage", "0.1").is_ok());
}

#[test]
fn a_selection_tasks_percentage_just_below_100_is_accepted() {
    assert!(with("--selection-tasks-percentage", "99.9").is_ok());
}

#[test]
fn a_part_that_is_held_out_has_no_flag_under_another_name() {
    assert_eq!(
        and(Some("--holdout=60".to_owned())).unwrap_err().kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn a_corroboration_tasks_percentage_of_0_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "0"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_of_100_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "100"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_below_0_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "-5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_above_100_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "100.5"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_that_is_not_a_number_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "NaN"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_written_with_a_percent_sign_is_an_error() {
    assert_eq!(
        refusal("--corroboration-tasks-percentage", "15%"),
        ErrorKind::ValueValidation
    );
}

#[test]
fn a_corroboration_tasks_percentage_that_is_refused_is_said_to_take_a_number_above_0_and_below_100()
{
    assert!(
        with("--corroboration-tasks-percentage", "100")
            .unwrap_err()
            .to_string()
            .contains("a number above 0 and below 100 is expected")
    );
}

#[test]
fn a_corroboration_tasks_percentage_is_the_number_given() {
    assert!(matches!(
        with("--corroboration-tasks-percentage", "15").unwrap().command,
        Command::Start(options) if options.corroboration_tasks_percentage == 15.0
    ));
}

#[test]
fn a_corroboration_tasks_percentage_with_a_decimal_point_is_the_number_given() {
    assert!(matches!(
        with("--corroboration-tasks-percentage", "12.5").unwrap().command,
        Command::Start(options) if options.corroboration_tasks_percentage == 12.5
    ));
}

#[test]
fn a_corroboration_tasks_percentage_just_above_0_is_accepted() {
    assert!(with("--corroboration-tasks-percentage", "0.1").is_ok());
}

#[test]
fn a_corroboration_tasks_percentage_just_below_100_is_accepted() {
    assert!(with("--corroboration-tasks-percentage", "99.9").is_ok());
}

#[test]
fn the_part_of_the_tasks_that_nothing_is_chosen_by_has_no_flag_under_another_name() {
    assert_eq!(
        and(Some("--corroboration-percentage=25".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_holdout_percentage_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--holdout-percentage=60".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn the_holdout_corroboration_percentage_is_no_flag_of_start() {
    assert_eq!(
        and(Some("--holdout-corroboration-percentage=50".to_owned()))
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}
