fn readme() -> String {
    test_inputs::README.to_owned()
}

#[test]
fn the_flag_of_the_benchmark_says_that_a_verifier_may_run_in_a_sandbox_of_its_own() {
    assert!(readme().contains(
        "Where the `task.toml` says that the verifier of the task runs separately, `cladenc` \
         runs the verifier in a sandbox of its own, and a task whose verifier is in the image \
         of that sandbox needs no `test.sh`."
    ));
}

#[test]
fn the_flag_of_the_benchmark_says_that_a_compose_file_in_the_tests_of_such_a_task_is_refused() {
    assert!(readme().contains(
        "So is a task whose verifier runs separately and whose `tests` directory holds a \
         Compose file."
    ));
}

#[test]
fn the_readme_says_when_a_verifier_image_is_built_and_removed() {
    assert!(readme().contains(
        "Where a task's `task.toml` says that its verifier runs separately and the task's \
         `tests` directory holds a Dockerfile, `cladenc` builds the verifier image with \
         the environments, from that Dockerfile with the `tests` directory as the build \
         context and under the name of the environment of the task with `-verifier` after it, \
         keeps what Docker printed in `verifier` beside what it printed for the environment of \
         the task, and removes the image with the environments."
    ));
}

#[test]
fn the_readme_says_that_an_image_named_for_a_verifier_has_to_be_on_the_machine() {
    assert!(readme().contains(
        "An image that the `task.toml` names for the verifier, as `docker_image` under \
         `verifier.environment`, has to be on the machine, as an image that it names for the \
         environment of the task has to be."
    ));
}

#[test]
fn the_readme_says_how_a_task_says_that_its_verifier_runs_separately_and_what_cladenc_then_does() {
    assert!(readme().contains(
        "Where the task's `task.toml` says that its verifier runs separately, with \
         `environment_mode` under `verifier` written as `separate` or with a table named \
         `environment` under `verifier`, `cladenc` removes the sandbox of the harness once \
         the paths are kept, and runs the verifier in a sandbox of its own."
    ));
}

#[test]
fn the_readme_says_what_a_verifier_sandbox_is_a_sandbox_of() {
    assert!(readme().contains(
        "That sandbox is of the image that `docker_image` under `verifier.environment` \
         names, or of the image built from the Dockerfile in the task's `tests` directory, \
         each of which holds the verifier at `/tests/test.sh`, or else of the environment of \
         the task as it is before a harness works in it, with the task's `tests` directory \
         copied in."
    ));
}

#[test]
fn the_readme_says_which_settings_a_verifier_sandbox_follows() {
    assert!(readme().contains(
        "The verifier sandbox follows the settings under `verifier.environment`, each \
         as `cladenc` obeys it under `environment`, or those under `environment` where the \
         task gives no such table."
    ));
}

#[test]
fn the_readme_says_that_a_verifier_of_its_own_finds_only_the_paths_that_were_kept() {
    assert!(readme().contains(
        "Each path that was kept is copied to the same path in the verifier sandbox, \
         so the verifier finds nothing else of what the harness left."
    ));
}

#[test]
fn the_readme_says_that_what_is_copied_to_a_verifier_is_not_trusted() {
    assert!(readme().contains(
        "No file that is copied in keeps a setuid or a setgid bit, and a symbolic link that \
         points outside the path it was kept with is left out, as is a file that is not a \
         regular file, a directory or a symbolic link."
    ));
}

#[test]
fn the_flag_of_the_most_retries_says_what_is_retried_and_that_0_is_taken() {
    assert!(readme().contains(
        "| `--max-verifier-health-check-retries <COUNT>` | Required | The most times a \
         verifier sandbox is brought up again, where its health check does not pass, 0 \
         or more."
    ));
}

#[test]
fn the_flag_of_the_most_retries_says_how_long_cladenc_waits_before_each_retry() {
    assert!(readme().contains(
        "Each retry is a new sandbox, and `cladenc` waits before it: the `interval_sec` of \
         the health check before the first retry, and twice as long before each retry after \
         that."
    ));
}

#[test]
fn the_flag_of_the_most_retries_says_what_becomes_of_a_trial_that_no_verifier_judged() {
    assert!(readme().contains(
        "Where the health check passes in no sandbox, the directed evolution stops and no \
         result is recorded for the trial. |\n"
    ));
}

#[test]
fn the_readme_says_that_a_verifier_waits_for_the_health_check_of_its_sandbox() {
    assert!(readme().contains(
        "Where those settings give a health check, `cladenc` runs it in the verifier \
         sandbox, and nothing else is done there until it has passed."
    ));
}

#[test]
fn the_readme_says_that_the_health_check_of_a_verifier_is_tried_again_in_a_new_sandbox() {
    assert!(readme().contains(
        "Where it does not pass, no verifier has judged what the harness left, so `cladenc` \
         does not record that the trial failed: it removes the \
         sandbox, waits, brings up a new verifier sandbox and runs the health check \
         there again, as many times as `--max-verifier-health-check-retries` gives."
    ));
}

#[test]
fn the_readme_says_how_long_cladenc_waits_before_a_retry_and_that_it_says_each() {
    assert!(readme().contains(
        "It waits the `interval_sec` of the health check before the first retry, or 5 \
         seconds where the task gives none, and twice as long before each retry after that, \
         and says each retry on standard error."
    ));
}

#[test]
fn the_readme_says_that_a_verifier_that_never_ran_stops_the_directed_evolution() {
    assert!(readme().contains(
        "Where the health check passes in no sandbox, the directed evolution stops with \
         exit status 1 and no result is recorded for the trial: `cladenc` names the \
         harness, the task, the number of tries and the directory that holds what the health \
         check printed the last time it ran, and `cladenc resume` carries out the trial \
         again."
    ));
}

#[test]
fn the_readme_names_the_three_results_of_a_trial() {
    assert!(readme().contains("The result of a trial is `pass`, `fail` or `inconclusive`."));
}

#[test]
fn the_readme_says_that_a_trial_passes_where_every_verdict_is_1() {
    assert!(
        readme()
            .contains("The trial passes where every verdict is 1: the harness completed the task.")
    );
}

#[test]
fn the_readme_says_that_a_trial_fails_where_a_verdict_is_not_1() {
    assert!(readme().contains(
        "Where the verifier writes a verdict that is not 1, the trial fails: the harness did \
         not complete the task."
    ));
}

#[test]
fn the_readme_says_that_a_trial_whose_verifier_gave_no_verdict_is_inconclusive() {
    assert!(readme().contains(
        "Where the verifier ends and leaves no verdict, or a file that cannot be read as one, \
         or where it runs past its time limit, the trial is inconclusive: nothing judged what \
         the harness left, so whether the harness completed the task is not known."
    ));
}

#[test]
fn the_readme_says_how_the_cause_of_an_inconclusive_trial_is_recorded_and_what_is_said() {
    assert!(readme().contains(
        "The cause of an inconclusive trial is recorded as `no_verdict` where the verifier \
         ended and as `timed_out` where the verifier ran past its time limit, and `cladenc` \
         says on standard error, at the end of a trial, whether the trial passed, failed or was \
         inconclusive."
    ));
}

#[test]
fn the_readme_says_that_nothing_tells_what_broke_a_verifier() {
    assert!(readme().contains(
        "The verifier may have broken by itself, or what the harness left may have broken it, \
         and `cladenc` has no way to tell which."
    ));
}

#[test]
fn the_readme_says_that_the_three_results_are_counted_apart() {
    assert!(readme().contains(
        "`cladenc report` and `cladenc status` count the trials that passed, those that failed \
         and those that were inconclusive apart."
    ));
}

#[test]
fn the_readme_says_that_an_inconclusive_trial_is_a_finished_trial() {
    assert!(readme().contains(
        "An inconclusive trial is a finished trial all the same: a completion rate is the part \
         of the finished trials that passed, and artificial selection goes by the finished \
         trials of a clade and by how many of them passed, so a harness gains nothing by \
         leaving what breaks a verifier."
    ));
}

#[test]
fn the_readme_says_what_cladenc_does_where_the_copy_of_logs_verifier_fails() {
    assert!(readme().contains(
        "Once the verifier has ended, `cladenc` copies `/logs/verifier` out of the sandbox and \
         reads the verdict in the copy. Where the copy fails, the verdict is not known, so no \
         result is recorded for the trial: the directed evolution stops with exit status 1, \
         `cladenc` names the harness and the task and says why the copy failed, and `cladenc \
         resume` carries out the trial again."
    ));
}

#[test]
fn the_readme_says_that_a_verifier_that_removed_logs_verifier_left_no_verdict() {
    assert!(readme().contains(
        "A verifier that removed `/logs/verifier` left nothing to copy, and so no verdict: the \
         trial is inconclusive, and its cause is recorded as `no_verdict`."
    ));
}

#[test]
fn the_readme_says_which_three_counts_the_report_gives_of_each_set_of_tasks() {
    assert!(readme().contains(
        "how many of its trials on the selection tasks passed, how many failed and how many \
         were inconclusive, the same three counts of the corroboration tasks and of the \
         non-holdout tasks, the three counts of the selection tasks for its clade, which is the \
         harness and all its descendants together,"
    ));
}

#[test]
fn the_readme_says_that_no_verifier_judges_part_of_what_the_harness_left() {
    assert!(readme().contains(
        "Where a path is in the sandbox of the harness and cannot be copied out of it, no result \
         is recorded for the trial, as said above, and no verifier is run: a verifier never \
         judges part of what the harness left."
    ));
}
