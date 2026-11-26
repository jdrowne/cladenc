use crate::cli::Cli;
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 14] = [
    ("--primogenitor", "/primogenitor"),
    ("--model", "a-model"),
    ("--model-url", "http://models.example/v1"),
    ("--dataset", "/dataset"),
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

fn without(left_out: &str) -> Result<Cli, clap::Error> {
    let options = OPTIONS
        .iter()
        .filter(|(option, _)| *option != left_out)
        .map(|(option, value)| format!("{option}={value}"));
    Cli::try_parse_from(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options),
    )
}

fn refusal_without(left_out: &str) -> ErrorKind {
    without(left_out).unwrap_err().kind()
}

#[test]
fn a_command_line_with_every_option_is_accepted() {
    assert!(without("").is_ok());
}

#[test]
fn leaving_out_the_primogenitor_is_an_error() {
    assert_eq!(
        refusal_without("--primogenitor"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_model_is_an_error() {
    assert_eq!(
        refusal_without("--model"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_address_of_the_model_is_an_error() {
    assert_eq!(
        refusal_without("--model-url"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_dataset_is_an_error() {
    assert_eq!(
        refusal_without("--dataset"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_docker_image_is_an_error() {
    assert_eq!(
        refusal_without("--diversification-docker-image"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_harness_run_command_is_an_error() {
    assert_eq!(
        refusal_without("--harness-run-command"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_outputs_directory_is_an_error() {
    assert_eq!(
        refusal_without("--outputs-dir"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_most_diversifications_is_an_error() {
    assert_eq!(
        refusal_without("--max-diversifications"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_limit_of_concurrent_sandboxes_is_an_error() {
    assert_eq!(
        refusal_without("--max-concurrent-sandboxes"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_most_retries_of_the_health_check_of_a_verifier_is_an_error() {
    assert_eq!(
        refusal_without("--max-verifier-health-check-retries"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_most_retries_of_the_health_check_of_a_task_is_an_error() {
    assert_eq!(
        refusal_without("--max-task-health-check-retries"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_diversification_timeout_is_an_error() {
    assert_eq!(
        refusal_without("--diversification-timeout"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_selection_tasks_percentage_is_an_error() {
    assert_eq!(
        refusal_without("--selection-tasks-percentage"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_corroboration_tasks_percentage_is_an_error() {
    assert_eq!(
        refusal_without("--corroboration-tasks-percentage"),
        ErrorKind::MissingRequiredArgument
    );
}
