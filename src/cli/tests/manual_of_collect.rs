use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_where_what_a_harness_left_in_logs_artifacts_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "What a harness left in /logs/artifacts is kept as a path that the task lists is, as \
         artifacts/logs/artifacts, though the task lists no such path, as in Harbor, and nothing \
         is kept where the harness left nothing there."
    ));
}

#[test]
fn the_manual_of_start_says_that_cladenc_makes_logs_artifacts_for_the_user_of_the_harness() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc makes the directory before the harness is started, as said above, and it belongs \
         to the user the harness runs as, so that a harness that does not run as root saves a file \
         there as well."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_task_that_lists_logs_artifacts_itself_gets() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task that lists /logs/artifacts itself, or a path that is kept at the same place, has \
         that entry obeyed in its place."
    ));
}

#[test]
fn the_manual_of_start_says_when_the_commands_under_collect_are_run() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml of a task lists commands under verifier.collect, which Harbor calls \
         collection hooks, cladenc runs each in the container of the harness once the harness has \
         ended or was stopped, before it keeps any path, in the order the task lists them."
    ));
}

#[test]
fn the_manual_of_start_says_how_a_command_under_collect_is_run() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A command is run as the harness is, under the same timer and in the working directory of \
         the environment, as the user its entry names as user, or the user of the environment \
         where it names none."
    ));
}

#[test]
fn the_manual_of_start_says_what_time_limit_a_command_under_collect_has() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "It has the time limit that its entry gives as timeout_sec, or 60 seconds where it gives \
         none."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_command_under_collect_that_fails_ends_no_trial() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A command that exits with another status than 0, or runs past its time limit, does not \
         end the trial: the next command is run, and the paths are kept as they then are."
    ));
}

#[test]
fn the_manual_of_start_says_where_what_a_command_under_collect_printed_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "collection-hooks in the directory of the trial keeps what each command printed, in a \
         directory named after the place of the command in the list, counted from 1."
    ));
}

#[test]
fn the_manual_of_start_names_the_commands_under_collect_among_what_cladenc_obeys() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "collect under verifier commands that are run in the container once the harness has ended, \
         before the paths under artifacts are kept"
    ));
}

#[test]
fn the_manual_of_start_names_collection_hooks_among_what_a_trial_keeps() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "collection-hooks what each command under verifier.collect printed, in a directory named \
         after its place in the list, where the task lists any"
    ));
}
