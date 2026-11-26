use crate::cli::Cli;
use clap::Parser;

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

fn said_of(command_line: impl Iterator<Item = String>) -> String {
    Cli::try_parse_from(command_line).unwrap_err().to_string()
}

fn said_of_start(options: impl Iterator<Item = String>) -> String {
    said_of(
        ["cladenc".to_owned(), "start".to_owned()]
            .into_iter()
            .chain(options),
    )
}

fn as_given() -> impl Iterator<Item = String> {
    OPTIONS
        .iter()
        .map(|(option, value)| format!("{option}={value}"))
}

fn said_with(more: &[&str]) -> String {
    said_of_start(as_given().chain(more.iter().map(|flag| (*flag).to_owned())))
}

fn said_without(left_out: &str) -> String {
    let start = format!("{left_out}=");
    said_of_start(as_given().filter(|option| !option.starts_with(&start)))
}

fn said_given(changed: &str, given: &str) -> String {
    let start = format!("{changed}=");
    said_of_start(as_given().map(|option| {
        if option.starts_with(&start) {
            format!("{changed}={given}")
        } else {
            option
        }
    }))
}

#[test]
fn quiet_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    assert!(said_with(&["--quiet"]).starts_with("error: unexpected argument '--quiet' found\n"));
}

#[test]
fn a_seed_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    assert!(said_with(&["--seed=7"]).starts_with("error: unexpected argument '--seed' found\n"));
}

#[test]
fn a_genealogy_growth_which_is_no_flag_of_start_is_refused_as_an_argument_not_expected() {
    assert!(
        said_with(&["--genealogy-growth=0.6"])
            .starts_with("error: unexpected argument '--genealogy-growth' found\n")
    );
}

#[test]
fn initial_diversifications_which_are_no_flag_of_start_are_refused_as_an_argument_not_expected() {
    assert!(
        said_with(&["--initial-diversifications=5"])
            .starts_with("error: unexpected argument '--initial-diversifications' found\n")
    );
}

#[test]
fn the_limit_of_concurrent_sandboxes_under_a_name_it_does_not_have_is_refused_for_the_name() {
    assert!(
        said_with(&["--max-concurrent-containers=8"])
            .starts_with("error: unexpected argument '--max-concurrent-containers' found\n")
    );
}

#[test]
fn the_dataset_under_the_name_benchmark_is_refused_for_the_name() {
    assert!(
        said_with(&["--benchmark=/dataset"])
            .starts_with("error: unexpected argument '--benchmark' found\n")
    );
}

#[test]
fn the_primogenitor_under_a_name_it_does_not_have_is_refused_for_the_name() {
    assert!(
        said_with(&["--progenitor=/primogenitor"])
            .starts_with("error: unexpected argument '--progenitor' found\n")
    );
}

#[test]
fn a_budget_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    assert!(
        said_with(&["--budget=800"]).starts_with("error: unexpected argument '--budget' found\n")
    );
}

#[test]
fn a_confidence_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    assert!(
        said_with(&["--confidence=99"])
            .starts_with("error: unexpected argument '--confidence' found\n")
    );
}

#[test]
fn a_command_line_without_the_most_diversifications_is_refused_for_the_flag_it_lacks() {
    assert!(said_without("--max-diversifications").contains("  --max-diversifications <COUNT>\n"));
}

#[test]
fn a_command_line_without_the_selection_tasks_percentage_is_refused_for_the_flag_it_lacks() {
    assert!(
        said_without("--selection-tasks-percentage")
            .contains("  --selection-tasks-percentage <NUMBER>\n")
    );
}

#[test]
fn a_command_line_without_the_corroboration_tasks_percentage_is_refused_for_the_flag_it_lacks() {
    assert!(
        said_without("--corroboration-tasks-percentage")
            .contains("  --corroboration-tasks-percentage <NUMBER>\n")
    );
}

#[test]
fn the_most_diversifications_being_0_is_refused_for_the_value() {
    assert!(
        said_given("--max-diversifications", "0")
            .starts_with("error: invalid value '0' for '--max-diversifications <COUNT>': ")
    );
}

#[test]
fn a_selection_tasks_percentage_of_100_is_refused_for_the_value() {
    assert!(
        said_given("--selection-tasks-percentage", "100").starts_with(
            "error: invalid value '100' for '--selection-tasks-percentage <NUMBER>': a number \
             above 0 and below 100 is expected\n"
        )
    );
}

#[test]
fn a_corroboration_tasks_percentage_of_100_is_refused_for_the_value() {
    assert!(
        said_given("--corroboration-tasks-percentage", "100").starts_with(
            "error: invalid value '100' for '--corroboration-tasks-percentage <NUMBER>': a \
             number above 0 and below 100 is expected\n"
        )
    );
}

#[test]
fn an_image_and_a_dockerfile_together_are_refused_by_the_flags_of_both() {
    assert!(
        said_with(&[
            "--diversification-dockerfile=/images/Dockerfile",
            "--diversification-docker-image-build-timeout=60"
        ])
        .starts_with(
            "error: the argument '--diversification-docker-image <IMAGE>' cannot be used with:\n  \
             --diversification-dockerfile <FILE>\n"
        )
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_is_refused_as_an_argument_that_is_not_expected() {
    let resume = ["cladenc", "resume", "--outputs-dir=/outputs", "--quiet"];
    assert!(
        said_of(resume.into_iter().map(str::to_owned))
            .starts_with("error: unexpected argument '--quiet' found\n")
    );
}
