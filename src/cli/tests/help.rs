use crate::cli::Cli;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

const WHAT_CLADENC_DOES: &str = "Carries out a directed evolution of an agentic coding harness, in \
                                 which harnesses procreate, each changing a copy of its own \
                                 source code, and are benchmarked";

const COMMANDS: [&[&str]; 5] = [
    &["cladenc"],
    &["cladenc", "report"],
    &["cladenc", "resume"],
    &["cladenc", "start"],
    &["cladenc", "status"],
];

fn printed_for(command: &[&str], flag: &str) -> String {
    let arguments = command.iter().copied().chain([flag]);
    Cli::try_parse_from(arguments).unwrap_err().to_string()
}

fn help_of_cladenc() -> String {
    printed_for(&["cladenc"], "--help")
}

fn help_of(sub_command: &str) -> String {
    printed_for(&["cladenc", sub_command], "--help")
}

fn description_of_the_package() -> String {
    test_inputs::PACKAGE
        .lines()
        .skip_while(|line| *line != "description: >-")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_long_flag_of_help_prints_what_the_short_flag_prints_for_every_command() {
    let printed = |flag: &str| -> Vec<String> {
        COMMANDS
            .iter()
            .map(|command| printed_for(command, flag))
            .collect()
    };
    assert_eq!(printed("--help"), printed("-h"));
}

#[test]
fn the_help_of_every_command_says_of_help_only_that_it_prints_help() {
    let printed: Vec<String> = COMMANDS
        .iter()
        .map(|command| printed_for(command, "--help"))
        .collect();
    assert!(printed.iter().all(|help| help.contains("Print help\n")));
}

#[test]
fn the_help_of_cladenc_names_the_manual_page_of_cladenc() {
    assert!(
        help_of_cladenc()
            .contains("\n\nThe manual page says the rest: man -M /opt/cladenc/share/man cladenc\n")
    );
}

#[test]
fn the_help_of_report_names_the_manual_page_of_report() {
    assert!(help_of("report").contains(
        "\n\nThe manual page says the rest: man -M /opt/cladenc/share/man cladenc-report\n"
    ));
}

#[test]
fn the_help_of_resume_names_the_manual_page_of_resume() {
    assert!(help_of("resume").contains(
        "\n\nThe manual page says the rest: man -M /opt/cladenc/share/man cladenc-resume\n"
    ));
}

#[test]
fn the_help_of_start_names_the_manual_page_of_start() {
    assert!(help_of("start").contains(
        "\n\nThe manual page says the rest: man -M /opt/cladenc/share/man cladenc-start\n"
    ));
}

#[test]
fn the_help_of_status_names_the_manual_page_of_status() {
    assert!(help_of("status").contains(
        "\n\nThe manual page says the rest: man -M /opt/cladenc/share/man cladenc-status\n"
    ));
}

#[test]
fn the_help_of_resume_does_not_show_the_label_of_a_directed_evolution() {
    assert!(!help_of("resume").contains("cladenc.directed-evolution"));
}

#[test]
fn the_help_of_start_names_the_flag_that_names_the_file_of_the_key() {
    assert!(help_of("start").contains("--model-key-file <FILE>"));
}

#[test]
fn the_help_of_start_names_the_flag_that_limits_the_containers_running_at_once() {
    assert!(help_of("start").contains("--max-concurrent-containers <COUNT>"));
}

#[test]
fn the_help_of_start_says_that_the_limit_is_the_most_containers_running_at_once() {
    assert!(help_of("start").contains("The most containers running at once"));
}

#[test]
fn the_help_of_start_names_the_two_flags_of_a_dockerfile() {
    let help = help_of("start");
    let flags = [
        "--procreation-dockerfile <FILE>",
        "--procreation-docker-image-build-timeout <SECONDS>",
    ];
    assert!(flags.iter().all(|flag| help.contains(flag)));
}

#[test]
fn the_help_of_start_names_the_flag_that_builds_a_harness() {
    assert!(help_of("start").contains("--harness-build-command <COMMAND>"));
}

#[test]
fn the_help_of_start_names_the_flag_that_gives_the_most_procreations() {
    assert!(help_of("start").contains("--max-procreations <COUNT>"));
}

#[test]
fn the_help_of_start_says_that_the_flag_gives_the_most_procreations_to_start() {
    assert!(help_of("start").contains("The most procreations to start"));
}

#[test]
fn the_help_of_start_names_the_flag_that_gives_the_most_retries_of_a_verifier_s_health_check() {
    assert!(help_of("start").contains("--max-verifier-health-check-retries <COUNT>"));
}

#[test]
fn the_help_of_start_says_what_the_flag_of_the_most_retries_counts() {
    assert!(help_of("start").contains("The most times a verifier container"));
}

#[test]
fn the_help_of_start_names_the_flag_that_gives_the_most_retries_of_a_task_s_health_check() {
    assert!(help_of("start").contains("--max-task-health-check-retries <COUNT>"));
}

#[test]
fn the_help_of_start_says_what_the_flag_of_the_most_retries_of_a_task_counts() {
    assert!(help_of("start").contains("The most times the container of a task"));
}

#[test]
fn the_help_of_start_does_not_show_how_a_harness_is_started() {
    assert!(!help_of("start").contains("sh -c <TIMER>"));
}

#[test]
fn the_help_of_every_sub_command_lists_its_flags_in_alphabetical_order() {
    let listed: Vec<Vec<String>> = Cli::command()
        .get_subcommands()
        .map(|sub_command| {
            sub_command
                .get_arguments()
                .map(|flag| flag.get_long().unwrap_or_default().to_owned())
                .collect()
        })
        .collect();
    let mut sorted = listed.clone();
    sorted.iter_mut().for_each(|flags| flags.sort());
    assert_eq!(listed, sorted);
}

#[test]
fn the_help_of_cladenc_opens_with_what_cladenc_does() {
    assert!(help_of_cladenc().starts_with(&format!("{WHAT_CLADENC_DOES}\n")));
}

#[test]
fn the_help_of_cladenc_does_not_say_that_the_harness_a_directed_evolution_makes_is_better() {
    assert!(!help_of_cladenc().contains("better"));
}

#[test]
fn the_help_of_cladenc_does_not_say_that_a_harness_modifies_itself() {
    assert!(!help_of_cladenc().contains("modify themselves"));
}

#[test]
fn the_description_of_the_package_is_the_line_the_help_of_cladenc_opens_with() {
    assert_eq!(
        description_of_the_package(),
        format!("{WHAT_CLADENC_DOES}.")
    );
}

#[test]
fn the_help_of_cladenc_names_start_as_the_sub_command_that_starts_a_directed_evolution() {
    assert!(help_of_cladenc().contains("  start   Start a new directed evolution"));
}

#[test]
fn the_help_of_cladenc_lists_no_sub_command_named_help() {
    assert!(!help_of_cladenc().contains("\n  help "));
}

#[test]
fn help_written_as_a_sub_command_is_refused() {
    assert_eq!(
        Cli::try_parse_from(["cladenc", "help"]).unwrap_err().kind(),
        ErrorKind::InvalidSubcommand
    );
}

#[test]
fn the_help_of_cladenc_says_that_report_is_for_a_directed_evolution_that_has_ended() {
    assert!(
        help_of_cladenc()
            .contains("\n  report  Print the report of a directed evolution that has ended, ")
    );
}

#[test]
fn the_help_of_cladenc_says_that_status_is_for_a_directed_evolution_that_has_not_ended() {
    assert!(help_of_cladenc().contains(
        "\n  status  Print the state so far of a directed evolution that has not ended, "
    ));
}

#[test]
fn the_help_of_report_names_the_flag_that_prints_a_json_object() {
    assert!(help_of("report").contains("--json"));
}

#[test]
fn the_help_of_status_names_the_flag_that_prints_a_json_object() {
    assert!(help_of("status").contains("--json"));
}

#[test]
fn the_help_of_status_names_the_flag_that_gives_the_outputs_directory() {
    assert!(help_of("status").contains("--outputs-dir <DIR>"));
}

#[test]
fn the_help_of_start_names_the_flag_that_names_the_benchmark() {
    assert!(help_of("start").contains("--benchmark <DIR>"));
}

#[test]
fn the_help_of_start_says_that_the_benchmark_flag_is_given_once_for_each_benchmark() {
    assert!(help_of("start").contains("given once for each benchmark"));
}

#[test]
fn the_help_of_start_names_the_flag_that_gives_the_selection_tasks_percentage() {
    assert!(help_of("start").contains("--selection-tasks-percentage <NUMBER>"));
}

#[test]
fn the_help_of_start_says_which_numbers_the_selection_tasks_percentage_takes() {
    assert!(help_of("start").contains("above 0 and below 100"));
}

#[test]
fn the_help_of_start_names_the_flag_that_gives_the_corroboration_tasks_percentage() {
    assert!(help_of("start").contains("--corroboration-tasks-percentage <NUMBER>"));
}

#[test]
fn the_help_of_start_says_what_the_corroboration_tasks_percentage_gives() {
    assert!(
        help_of("start")
            .contains("The percentage of each benchmark's tasks that are corroboration tasks")
    );
}

#[test]
fn the_help_of_start_says_what_the_selection_tasks_percentage_gives() {
    assert!(
        help_of("start")
            .contains("The percentage of each benchmark's tasks that are selection tasks")
    );
}
