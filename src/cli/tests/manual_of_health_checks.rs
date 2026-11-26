use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_names_the_health_check_among_what_cladenc_obeys() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "healthcheck under environment a command that has to pass before the harness is started, \
         and how often it is run"
    ));
}

#[test]
fn the_manual_of_start_says_when_and_as_whom_a_health_check_is_run() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml of a task gives a health check, as command under \
         environment.healthcheck, cladenc runs that command in the sandbox before it copies the \
         source code of the harness in, under the same timer, as the user of the environment and \
         in its working directory."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_harness_is_started_once_the_health_check_passes() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("The harness is started only once the command has exited with status 0.")
    );
}

#[test]
fn the_manual_of_start_says_what_time_limit_a_run_of_a_health_check_has() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A run of the command has the time limit timeout_sec, or 30 seconds where the task gives \
         none."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_health_check_is_run_again() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A run that exits with another status, or runs past its time limit, has failed, and the \
         command is run again after interval_sec, or 5 seconds where the task gives none."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_health_check_has_not_passed() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The health check has not passed once the command has failed as many times as retries \
         says, or 3 times where the task gives none."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_failure_in_the_start_period_is_not_counted() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A failure within start_period_sec of the first run, or 0 seconds where the task gives \
         none, is not counted, and the command is then run again after start_interval_sec, or 5 \
         seconds where the task gives none."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_harness_that_was_never_started_is_given_no_result() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the health check of a task does not pass, the harness was never started, so cladenc \
         does not record that the harness did not complete the task."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_health_check_of_a_task_is_tried_again_in_a_new_sandbox() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "It removes the sandbox, waits, brings up a new sandbox of the task and runs the \
         health check there again, as many times as --max-task-health-check-retries gives."
    ));
}

#[test]
fn the_manual_of_start_says_how_long_cladenc_waits_before_each_retry_of_a_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "gives. Before the first retry it waits interval_sec of the health check, or 5 seconds \
         where the task gives none, and before each retry after that twice as long as before the \
         last: 5 seconds, then 10, then 20."
    ));
}

#[test]
fn the_manual_of_start_says_that_each_retry_of_the_health_check_of_a_task_is_said() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("then 20. It says each retry on standard error. Where the health check")
    );
}

#[test]
fn the_manual_of_start_says_that_a_task_that_is_never_ready_stops_the_directed_evolution() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the health check passes in no sandbox, the directed evolution stops with exit \
         status 1, and no result is recorded for the trial."
    ));
}

#[test]
fn the_manual_of_start_says_what_cladenc_names_of_a_harness_that_was_never_started() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc then names the harness and the task, says that the health check of the task did \
         not pass and in how many tries, and names the directory that holds what the health check \
         printed the last time it ran."
    ));
}

#[test]
fn the_manual_of_start_says_where_what_the_health_check_of_a_task_printed_stays() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "ran. That directory is under running in the outputs directory, and stays there until \
         cladenc resume carries out the trial again. ARTIFACTS Where the task.toml of a task \
         lists paths under artifacts"
    ));
}

#[test]
fn the_manual_of_start_does_not_say_that_a_harness_that_was_not_started_did_not_complete() {
    assert!(!manual_on_one_line("cladenc-start").contains("the harness is not started"));
}

#[test]
fn the_manual_of_start_says_which_settings_of_a_health_check_are_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Of a health check, timeout_sec is refused where it is not a number of seconds above 0, \
         retries where they are not a number above 0, and interval_sec, start_period_sec and \
         start_interval_sec where they are not a number of seconds, 0 or more."
    ));
}

#[test]
fn the_manual_of_start_says_where_what_a_health_check_printed_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "health-check the standard output and the standard error of the last run of the health \
         check, in stdout and stderr, where the task gives one"
    ));
}

#[test]
fn the_manual_of_start_names_no_trial_whose_harness_was_not_started() {
    assert!(!manual_on_one_line("cladenc-start").contains("not_started"));
}
