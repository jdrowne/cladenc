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
    assert!(
        long_help_of("start")
            .contains("\n    docker build --tag <NAME> --label cladenc.task=<TASK> <DIR>\n")
    );
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
        "The build has no time limit, and environments/<BENCHMARK>/<TASK> in the\noutputs \
         directory keeps what Docker printed."
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
