use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of(sub_command: &str) -> String {
    let mut cladenc = Cli::command();
    let sub_command = cladenc.find_subcommand_mut(sub_command).unwrap();
    sub_command.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_what_a_task_with_a_grader_of_its_own_holds_in_its_tests() {
    assert!(long_help_of("start").contains(
        "A task whose grader runs in a container of its own, as said below, is\nrefused as \
         well for a Compose file in its tests directory. Where the image\nof that container \
         holds the grader, the task needs no test.sh."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_grader_that_runs_separately_has_a_container_of_its_own() {
    assert!(long_help_of("start").contains(
        "Where the task.toml of a task says that its grader runs separately, cladenc\nruns \
         the grader in a container of its own, which holds nothing of the\ncontainer of the \
         harness but the paths the task lists under artifacts."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_task_says_that_its_grader_runs_separately() {
    assert!(long_help_of("start").contains(
        "A\ntask says so with environment_mode under verifier, written as separate, or\nwith \
         a table named environment under verifier, which says so by being there."
    ));
}

#[test]
fn the_long_help_of_start_says_which_graders_run_in_the_container_of_the_harness() {
    assert!(long_help_of("start").contains(
        "A task that gives neither, or writes environment_mode as shared, has its\ngrader run \
         in the container of the harness, as said above."
    ));
}

#[test]
fn the_long_help_of_start_lists_what_the_container_of_a_grader_is_a_container_of() {
    assert!(long_help_of("start").contains(
        "The container of the grader is a container of the first of these that the\ntask \
         gives:\n\n  1. the image that docker_image under verifier.environment names, \
         which\n     has to be on the machine, as an image that a task names for its\n     \
         environment has to be\n  2. an image that cladenc builds from the Dockerfile in the \
         tests\n     directory of the task, with that directory as the build context\n  3. \
         the environment of the task, as it is before a harness works in it\n"
    ));
}

#[test]
fn the_long_help_of_start_says_which_images_hold_the_grader_and_which_are_given_the_tests() {
    assert!(long_help_of("start").contains(
        "An image of 1 or 2 holds the grader, at /tests/test.sh: cladenc copies no\ntests \
         directory into its container and makes nothing at /tests, and the task\nneeds no \
         test.sh. Into a container of 3 the tests directory of the task is\ncopied, and /tests \
         is a symbolic link to the copy, as said above."
    ));
}

#[test]
fn the_long_help_of_start_says_when_the_image_of_a_grader_is_built_and_under_what_name() {
    assert!(long_help_of("start").contains(
        "cladenc\nbuilds an image of 2 when it builds the environments of the tasks, and as \
         it\nbuilds those, under the name of the environment of the task with -grader\nafter \
         it, and removes it when it removes them."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_docker_printed_for_the_image_of_a_grader_is_kept() {
    assert!(long_help_of("start").contains(
        "environments/<BENCHMARK>/<TASK>/grader in the outputs directory keeps what\nDocker \
         printed."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_image_of_a_grader_that_is_missing_stops_with_status_2() {
    assert!(long_help_of("start").contains(
        "An image of 1 that is not on the machine, and an image of 2\nthat is not built, stop \
         the directed evolution with exit status 2."
    ));
}

#[test]
fn the_long_help_of_start_says_which_settings_the_container_of_a_grader_follows() {
    assert!(long_help_of("start").contains(
        "Where the task.toml holds the table environment under verifier, the\ncontainer of \
         the grader follows that table and nothing of environment:\nworkdir, env, cpus, \
         memory_mb, healthcheck and build_timeout_sec are obeyed\nthere as they are under \
         environment, and a setting the table leaves out is\nleft out."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_grader_with_no_settings_of_its_own_follows() {
    assert!(long_help_of("start").contains(
        "Where the task.toml holds no such table, the container of the\ngrader follows \
         environment. What is refused under environment is refused\nunder \
         verifier.environment as well."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_container_of_the_harness_is_removed_before_the_grader_s() {
    assert!(long_help_of("start").contains(
        "Once the harness has ended or was stopped, and the paths under artifacts are\nkept, \
         cladenc removes the container of the harness and brings up that of the\ngrader, so \
         that an attempt is one container at a time."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_health_check_of_a_grader_that_does_not_pass_means() {
    assert!(long_help_of("start").contains(
        "Where the settings of\nthe grader give a health check, cladenc runs it there as it \
         runs that of the\ntask: where it does not pass, the grader is not run, and the \
         harness did not\ncomplete the task."
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_kept_paths_are_copied_in_the_container_of_a_grader() {
    assert!(long_help_of("start").contains(
        "each path that was kept is copied to the same\npath in the container of the \
         grader, the path of its entry and not\ndestination, without what exclude left out. A \
         directory takes the place of\nwhat the image holds at that path."
    ));
}

#[test]
fn the_long_help_of_start_says_that_logs_is_the_same_link_in_the_container_of_a_grader() {
    assert!(long_help_of("start").contains(
        "/logs is then made the symbolic link that it is in the\ncontainer of the harness, and \
         each path that was kept is copied"
    ));
}

#[test]
fn the_long_help_of_start_says_that_what_is_copied_to_a_grader_is_not_trusted() {
    assert!(long_help_of("start").contains(
        "What is copied in is not trusted. No file keeps a setuid or a setgid bit. A\nfile \
         that is not a regular file, a directory or a symbolic link is left out,\nand so is a \
         symbolic link that points outside the path it was kept with.\nWhat is copied in \
         belongs to the user the grader runs as."
    ));
}

#[test]
fn the_long_help_of_start_says_how_the_grader_is_run_in_a_container_of_its_own() {
    assert!(long_help_of("start").contains(
        "Then the tests directory is copied in where the image holds no \
         grader,\n/logs/verifier is emptied, and the grader is run as said above, in \
         the\nworking directory and as the user of its own container, unless the \
         task.toml\nnames a user for the grader. Its time limit is counted from then."
    ));
}

#[test]
fn the_long_help_of_start_says_how_long_the_container_of_a_grader_stays() {
    assert!(long_help_of("start").contains(
        "The\ncontainer of the grader is started as\n\n    sleep <SECONDS>\n\n  <SECONDS>  \
         the time limit of the grader, the longest the health check\n             of the \
         grader can take, and ten minutes more\n"
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_image_of_a_grader_has_to_hold() {
    assert!(long_help_of("start").contains(
        "and is removed once the grader has ended. The image of a grader has to hold\nwhat \
         the environment of a task has to: sh, sleep, rm, mkdir, chown and ln."
    ));
}

#[test]
fn the_long_help_of_start_names_the_mode_of_the_grader_among_what_cladenc_obeys() {
    assert!(long_help_of("start").contains(
        "  environment_mode under          separate where the grader runs in a\n  \
         verifier                        container of its own, and shared \
         where\n                                  it runs in that of the harness\n"
    ));
}

#[test]
fn the_long_help_of_start_names_the_environment_of_the_grader_among_what_cladenc_obeys() {
    assert!(long_help_of("start").contains(
        "  environment under verifier      what environment says of the \
         container\n                                  of the harness, said of the \
         container\n                                  of the grader\n"
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_health_check_of_a_grader_is_kept() {
    assert!(long_help_of("start").contains(
        "left in /logs/verifier, its score among them; where the\n                 grader ran \
         in a container of its own with a health check,\n                 what the last run \
         of that printed, in health-check\n"
    ));
}
