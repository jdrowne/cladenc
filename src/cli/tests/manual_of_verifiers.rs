use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_what_a_task_with_a_verifier_of_its_own_holds_in_its_tests() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task whose verifier runs in a container of its own, as said below, is refused as well \
         for a Compose file in its tests directory. Where the image of that container holds the \
         verifier, the task needs no test.sh."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_that_runs_separately_has_a_container_of_its_own() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml of a task says that its verifier runs separately, cladenc runs the \
         verifier in a container of its own, which holds nothing of the container of the harness \
         but the paths the task lists under artifacts."
    ));
}

#[test]
fn the_manual_of_start_says_how_a_task_says_that_its_verifier_runs_separately() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task says so with environment_mode under verifier, written as separate, or with a table \
         named environment under verifier, which says so by being there."
    ));
}

#[test]
fn the_manual_of_start_says_which_verifiers_run_in_the_container_of_the_harness() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task that gives neither, or writes environment_mode as shared, has its verifier run in \
         the container of the harness, as said above."
    ));
}

#[test]
fn the_manual_of_start_lists_what_a_verifier_container_is_a_container_of() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The verifier container is a container of the first of these that the task gives: 1. \
         the image that docker_image under verifier.environment names, which has to be on the \
         machine, as an image that a task names for its environment has to be 2. an image that \
         cladenc builds from the Dockerfile in the tests directory of the task, with that \
         directory as the build context 3. the environment of the task, as it is before a harness \
         works in it"
    ));
}

#[test]
fn the_manual_of_start_says_which_images_hold_the_verifier_and_which_are_given_the_tests() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "An image of 1 or 2 holds the verifier, at /tests/test.sh: cladenc copies no tests \
         directory into its container and makes nothing at /tests, and the task needs no test.sh. \
         Into a container of 3 the tests directory of the task is copied, and /tests is a symbolic \
         link to the copy, as said above."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_verifier_image_is_built_and_under_what_name() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc builds an image of 2 when it builds the environments of the tasks, and as it \
         builds those, under the name of the environment of the task with -verifier after it, and \
         removes it when it removes them."
    ));
}

#[test]
fn the_manual_of_start_says_where_what_docker_printed_for_a_verifier_image_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "environments/<BENCHMARK>/<TASK>/verifier in the outputs directory keeps what Docker \
         printed."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_image_that_is_missing_stops_with_status_2() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "An image of 1 that is not on the machine, and an image of 2 that is not built, stop the \
         directed evolution with exit status 2."
    ));
}

#[test]
fn the_manual_of_start_says_which_settings_a_verifier_container_follows() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml holds the table environment under verifier, the verifier \
         container follows that table and nothing of environment: workdir, env, cpus, memory_mb, \
         healthcheck and build_timeout_sec are obeyed there as they are under environment, and a \
         setting the table leaves out is left out."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_verifier_with_no_settings_of_its_own_follows() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml holds no such table, the verifier container follows environment. \
         What is refused under environment is refused under verifier.environment as well."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_container_of_the_harness_is_removed_before_the_verifier_s() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Once the harness has ended or was stopped, and the paths under artifacts are kept, \
         cladenc removes the container of the harness and brings up that of the verifier, so that \
         a trial is one container at a time."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_waits_for_the_health_check_of_its_container() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the settings of the verifier give a health check, cladenc runs it there as it runs \
         that of the task, and nothing else is done in the container until the health check has \
         passed."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_that_never_ran_judged_nothing() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the health check of the verifier container does not pass, no verifier has judged \
         what the harness left, so cladenc does not record that the harness did not complete the \
         task."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_health_check_of_a_verifier_is_tried_again_in_a_new_container()
{
    assert!(manual_on_one_line("cladenc-start").contains(
        "It removes the container, waits, brings up a new verifier container and runs the \
         health check there again, as many times as --max-verifier-health-check-retries gives."
    ));
}

#[test]
fn the_manual_of_start_says_how_long_cladenc_waits_before_each_retry() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Before the first retry it waits interval_sec of the health check, or 5 seconds where the \
         task gives none, and before each retry after that twice as long as before the last: 5 \
         seconds, then 10, then 20."
    ));
}

#[test]
fn the_manual_of_start_says_that_each_retry_is_said() {
    assert!(manual_on_one_line("cladenc-start").contains("It says each retry on standard error."));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_that_never_ran_stops_the_directed_evolution() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the health check passes in no container, the directed evolution stops with exit \
         status 1, and no result is recorded for the trial."
    ));
}

#[test]
fn the_manual_of_start_says_what_cladenc_names_when_a_verifier_never_ran() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc then names the harness and the task, says that the health check of the verifier \
         container did not pass and in how many tries, and names the directory that holds what \
         the health check printed the last time it ran. That directory is under running in the \
         outputs directory, and stays there until cladenc resume carries out the trial again."
    ));
}

#[test]
fn the_manual_of_start_says_where_the_kept_paths_are_copied_in_a_verifier_container() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "each path that was kept is copied to the same path in the verifier container, the \
         path of its entry and not destination, without what exclude left out. A directory takes \
         the place of what the image holds at that path."
    ));
}

#[test]
fn the_manual_of_start_says_that_logs_is_the_same_link_in_a_verifier_container() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/logs is then made the symbolic link that it is in the container of the harness, and each \
         path that was kept is copied"
    ));
}

#[test]
fn the_manual_of_start_says_that_what_is_copied_to_a_verifier_is_not_trusted() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "What is copied in is not trusted. No file keeps a setuid or a setgid bit. A file that is \
         not a regular file, a directory or a symbolic link is left out, and so is a symbolic link \
         that points outside the path it was kept with. What is copied in belongs to the user the \
         verifier runs as."
    ));
}

#[test]
fn the_manual_of_start_says_how_the_verifier_is_run_in_a_container_of_its_own() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Then the tests directory is copied in where the image holds no verifier, /logs/verifier \
         is emptied, and the verifier is run as said above, in the working directory and as the \
         user of its own container, unless the task.toml names a user for the verifier. Its time \
         limit is counted from then."
    ));
}

#[test]
fn the_manual_of_start_says_how_long_a_verifier_container_stays() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The verifier container is started as sleep <SECONDS> <SECONDS> the time limit of the \
         verifier, the longest the health check of the verifier can take, and ten minutes more"
    ));
}

#[test]
fn the_manual_of_start_says_what_a_verifier_image_has_to_hold() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and is removed once the verifier has ended. A verifier image has to hold what the \
         environment of a task has to: sh, sleep, rm, mkdir, chown and ln."
    ));
}

#[test]
fn the_manual_of_start_names_the_mode_of_the_verifier_among_what_cladenc_obeys() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "environment_mode under verifier separate where the verifier runs in a container of its \
         own, and shared where it runs in that of the harness"
    ));
}

#[test]
fn the_manual_of_start_names_the_verifier_environment_among_what_cladenc_obeys() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "environment under verifier what environment says of the container of the harness, said of \
         the verifier container"
    ));
}

#[test]
fn the_manual_of_start_says_where_the_health_check_of_a_verifier_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "left in /logs/verifier, its score among them; where the verifier ran in a container of \
         its own with a health check, what the last run of that printed, in health-check"
    ));
}
