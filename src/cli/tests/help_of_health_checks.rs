use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of_start() -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    start.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_names_the_health_check_among_what_cladenc_obeys() {
    assert!(long_help_of_start().contains(
        "  healthcheck under environment   a command that has to pass before the\n                                  \
         harness is started, and how often it\n                                  is run\n"
    ));
}

#[test]
fn the_long_help_of_start_says_when_and_as_whom_a_health_check_is_run() {
    assert!(long_help_of_start().contains(
        "Where the task.toml of a task gives a health check, as command under\n\
         environment.healthcheck, cladenc runs that command in the container before\nit copies \
         the source code of the harness in, under the same timer, as the\nuser of the \
         environment and in its working directory."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_harness_is_started_once_the_health_check_passes() {
    assert!(
        long_help_of_start()
            .contains("The harness is\nstarted only once the command has exited with status 0.")
    );
}

#[test]
fn the_long_help_of_start_says_what_time_limit_a_run_of_a_health_check_has() {
    assert!(long_help_of_start().contains(
        "A run of the\ncommand has the time limit timeout_sec, or 30 seconds where the task \
         gives\nnone."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_health_check_is_run_again() {
    assert!(long_help_of_start().contains(
        "A run that exits with another status, or runs past its time limit,\nhas failed, and \
         the command is run again after interval_sec, or 5 seconds\nwhere the task gives none."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_health_check_has_not_passed() {
    assert!(long_help_of_start().contains(
        "The health check has not passed once the\ncommand has failed as many times as retries \
         says, or 3 times where the\ntask gives none."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_failure_in_the_start_period_is_not_counted() {
    assert!(long_help_of_start().contains(
        "A failure within start_period_sec of the first run, or 0\nseconds where the task \
         gives none, is not counted, and the command is then\nrun again after \
         start_interval_sec, or 5 seconds where the task gives none."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_health_check_that_does_not_pass_gives() {
    assert!(long_help_of_start().contains(
        "Where the health check does not pass, the harness is not started and the\ngrader is \
         not run: the harness did not complete the task, and cladenc says\nthat the environment \
         of the task did not pass its health check."
    ));
}

#[test]
fn the_long_help_of_start_says_which_settings_of_a_health_check_are_refused() {
    assert!(long_help_of_start().contains(
        "Of a health check, timeout_sec is refused where it is\nnot a number of seconds above \
         0, retries where they are not a number above\n0, and interval_sec, start_period_sec \
         and start_interval_sec where they are\nnot a number of seconds, 0 or more."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_a_health_check_printed_is_kept() {
    assert!(long_help_of_start().contains(
        "  health-check   the standard output and the standard error of the last\n                 \
         run of the health check, in stdout and stderr, where the\n                 task gives \
         one\n"
    ));
}

#[test]
fn the_long_help_of_start_says_what_an_attempt_whose_harness_was_not_started_keeps() {
    assert!(long_help_of_start().contains(
        "\nAn attempt whose harness was not started keeps result, ending,\nspecification and \
         health-check, and nothing else.\n"
    ));
}
