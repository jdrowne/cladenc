use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of(sub_command: &str) -> String {
    let mut cladenc = Cli::command();
    let sub_command = cladenc.find_subcommand_mut(sub_command).unwrap();
    sub_command.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_when_the_environment_of_every_task_is_built() {
    assert!(long_help_of("start").contains(
        "cladenc builds the environment of every task of every benchmark when the\ndirected \
         evolution is started, and again each time it is resumed, before\nit brings up any \
         container, as"
    ));
}

#[test]
fn the_long_help_of_start_shows_how_docker_is_asked_to_build_an_environment() {
    assert!(long_help_of("start").contains(
        "\n    docker build --tag <NAME> --label cladenc.task=<TASK> \\\n        --label \
         cladenc.directed-evolution=<PATH> <DIR>\n"
    ));
}

#[test]
fn the_long_help_of_start_says_what_path_the_label_of_an_environment_holds() {
    assert!(long_help_of("start").contains(
        "  <PATH>  the path of the outputs directory, with every symbolic link in\n          it \
         resolved\n"
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_labels_of_an_environment_are() {
    assert!(long_help_of("start").contains(
        "The two labels are Docker labels, each a name and a value that Docker\nkeeps with an \
         image. The second is the label that every container of the\ndirected evolution \
         carries."
    ));
}

#[test]
fn the_long_help_of_start_says_that_two_directed_evolutions_build_two_images_of_one_task() {
    assert!(long_help_of("start").contains(
        "It makes the environments of two directed\nevolutions two images, even where both \
         are built from one task of one\nbenchmark, so that removing those of one leaves \
         those of the other on the\nmachine."
    ));
}

#[test]
fn the_long_help_of_start_says_how_cladenc_names_the_environment_of_a_task() {
    assert!(long_help_of("start").contains(
        "  <NAME>  the name cladenc gives the environment: cladenc-environments-\n          and \
         sixteen random hexadecimal digits, the same for every task\n          of the directed \
         evolution, then a colon and the place of the\n          task among the tasks of the \
         directed evolution, counted from 0\n"
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_environment_is_built_from_the_copy_of_its_task() {
    assert!(long_help_of("start").contains(
        "  <DIR>   the environment directory of the copy of the task in the\n          outputs \
         directory, the build context of its Dockerfile\n"
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_docker_printed_for_an_environment_is_kept() {
    assert!(long_help_of("start").contains(
        "environments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker\nprinted."
    ));
}

#[test]
fn the_long_help_of_start_says_what_time_limit_the_build_of_an_environment_has() {
    assert!(long_help_of("start").contains(
        "The build has the time limit that the task.toml of the task gives, \
         as\nbuild_timeout_sec under environment, or 600 seconds where it gives none."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_environment_that_is_not_built_stops_with_exit_status_2() {
    assert!(long_help_of("start").contains(
        "A task whose environment is\nnot built stops the directed evolution with exit status \
         2, and\n`cladenc start` leaves it no journal."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_base_image_of_an_environment_has_to_be_on_the_machine() {
    assert!(long_help_of("start").contains(
        "The image that the Dockerfile\nof a task starts from has to be on the machine: cladenc \
         does not ask\nDocker to fetch it and cannot forbid it, so Docker fetches a base \
         image\nthat is not on the machine, from the registry of its name."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_it_builds_the_environments_again_under_the_same_names() {
    assert!(long_help_of("resume").contains(
        "Once the containers are removed, `cladenc resume` builds the environment\nof every task \
         again, from the copy of the task in the outputs directory\nand under the name it was \
         given when the directed evolution was started,\nbefore it brings up any container."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_the_base_image_of_an_environment_has_to_be_on_the_machine() {
    assert!(long_help_of("resume").contains(
        "The image that the Dockerfile of each\ntask starts from has to be on the machine: \
         Docker fetches one that is\nnot, from the registry of its name."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_an_environment_that_is_not_built_stops_with_exit_status_2() {
    assert!(long_help_of("resume").contains(
        "A task whose environment is not built\nstops `cladenc resume` with exit status 2, \
         and\nenvironments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker\nprinted."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_environments_cladenc_built_are_removed_when_it_ends() {
    assert!(long_help_of("start").contains(
        "cladenc removes the environments it built once `cladenc start` or\n`cladenc resume` \
         ends, whether the directed evolution finished, stopped or\ncould not go on, as\n"
    ));
}

#[test]
fn the_long_help_of_start_shows_how_docker_is_asked_to_remove_an_environment() {
    assert!(long_help_of("start").contains("\n    docker image rm --force <NAME>...\n"));
}

#[test]
fn the_long_help_of_start_says_that_an_image_a_task_names_is_left_alone() {
    assert!(long_help_of("start").contains(
        "An image that the\ntask.toml of a task names is not one cladenc built, and is left \
         alone."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_cladenc_killed_outright_removes_no_environment() {
    assert!(long_help_of("start").contains(
        "A\ncladenc that was killed outright removes nothing: `cladenc resume` builds\nthe \
         environments again, and removes them when it ends."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_stop_before_the_journal_does() {
    assert!(long_help_of("start").contains(
        "`cladenc start`\nthen exits with status 2 and says that the directed evolution was \
         stopped\nbefore it was started: `cladenc resume` has nothing to continue, and \
         the\noutputs directory is removed by hand before it is named to `cladenc \
         start`\nagain."
    ));
}
