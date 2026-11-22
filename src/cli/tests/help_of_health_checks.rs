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
fn the_long_help_of_start_says_that_a_harness_that_was_never_started_is_given_no_result() {
    assert!(long_help_of_start().contains(
        "\nWhere the health check of a task does not pass, the harness was never\nstarted, so \
         cladenc does not record that the harness did not complete the\ntask."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_health_check_of_a_task_is_tried_again_in_a_new_container() {
    assert!(long_help_of_start().contains(
        "It removes the container, waits, brings up a new container of the\ntask and runs the \
         health check there again, as many times as\n--max-task-health-check-retries gives."
    ));
}

#[test]
fn the_long_help_of_start_says_how_long_cladenc_waits_before_each_retry_of_a_task() {
    assert!(long_help_of_start().contains(
        "gives. Before the first retry it waits\ninterval_sec of the health check, or 5 seconds \
         where the task gives none,\nand before each retry after that twice as long as before \
         the last: 5\nseconds, then 10, then 20."
    ));
}

#[test]
fn the_long_help_of_start_says_that_each_retry_of_the_health_check_of_a_task_is_said() {
    assert!(
        long_help_of_start()
            .contains("then 20. It says each retry on standard error. Where the\nhealth check")
    );
}

#[test]
fn the_long_help_of_start_says_that_a_task_that_is_never_ready_stops_the_directed_evolution() {
    assert!(long_help_of_start().contains(
        "Where the\nhealth check passes in no container, the directed evolution stops with \
         exit\nstatus 1, and no result is recorded for the attempt."
    ));
}

#[test]
fn the_long_help_of_start_says_what_cladenc_names_of_a_harness_that_was_never_started() {
    assert!(long_help_of_start().contains(
        "cladenc then names the\nharness and the task, says that the health check of the task \
         did not pass\nand in how many tries, and names the directory that holds what the \
         health\ncheck printed the last time it ran."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_the_health_check_of_a_task_printed_stays() {
    assert!(long_help_of_start().contains(
        "ran. That directory is under running in the\noutputs directory, and stays there until \
         `cladenc resume` carries out the\nattempt again.\n\nWhere the task.toml of a task \
         lists paths under artifacts"
    ));
}

#[test]
fn the_long_help_of_start_does_not_say_that_a_harness_that_was_not_started_did_not_complete() {
    assert!(!long_help_of_start().contains("the harness is not started"));
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
fn the_long_help_of_start_names_no_attempt_whose_harness_was_not_started() {
    assert!(!long_help_of_start().contains("not_started"));
}
