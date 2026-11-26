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
         does not record that the harness did not complete the task: it removes the \
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
fn the_readme_says_that_a_verdict_that_is_not_1_and_no_verdict_are_told_apart() {
    assert!(readme().contains(
        "Where the verifier writes a verdict that is not 1, the harness did not complete the \
         task. Where the verifier ends and leaves no verdict, or a file that cannot be read as \
         one, the harness did not complete the task either, and `cladenc` tells the two apart: \
         it says which of the two it was on standard error at the end of the trial, and records \
         the first as `not_completed` and the second as `no_verdict`."
    ));
}

#[test]
fn the_readme_says_that_a_task_whose_verifier_left_no_verdict_counts_as_not_completed() {
    assert!(readme().contains(
        "The verifier may have broken by itself, or what the harness left may have broken it, \
         and `cladenc` has no way to tell which, so a task whose verifier left no verdict \
         counts among the tasks the harness did not complete, in artificial selection and in \
         what `cladenc report` and `cladenc status` print."
    ));
}
