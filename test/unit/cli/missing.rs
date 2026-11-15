use cladenc::cli::Cli;
use clap::Parser;
use clap::error::ErrorKind;

const OPTIONS: [(&str, &str); 16] = [
    ("--progenitor", "/progenitor"),
    ("--model", "a-model"),
    ("--model-url", "http://models.example/v1"),
    ("--benchmark", "/benchmark"),
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
fn leaving_out_the_progenitor_is_an_error() {
    assert_eq!(
        refusal_without("--progenitor"),
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
fn leaving_out_the_benchmark_is_an_error() {
    assert_eq!(
        refusal_without("--benchmark"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_docker_image_is_an_error() {
    assert_eq!(
        refusal_without("--docker-image"),
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
fn leaving_out_the_evaluator_is_an_error() {
    assert_eq!(
        refusal_without("--evaluator"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_run_directory_is_an_error() {
    assert_eq!(
        refusal_without("--run-dir"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_budget_is_an_error() {
    assert_eq!(
        refusal_without("--budget"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_genealogy_growth_is_an_error() {
    assert_eq!(
        refusal_without("--genealogy-growth"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_confidence_is_an_error() {
    assert_eq!(
        refusal_without("--confidence"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_initial_procreations_is_an_error() {
    assert_eq!(
        refusal_without("--initial-procreations"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_workers_is_an_error() {
    assert_eq!(
        refusal_without("--workers"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_procreation_timeout_is_an_error() {
    assert_eq!(
        refusal_without("--procreation-timeout"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_evaluation_timeout_is_an_error() {
    assert_eq!(
        refusal_without("--evaluation-timeout"),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn leaving_out_the_seed_is_an_error() {
    assert_eq!(
        refusal_without("--seed"),
        ErrorKind::MissingRequiredArgument
    );
}
