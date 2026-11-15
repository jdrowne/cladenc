use cladenc::cli::Cli;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

fn help_of_start(long: bool) -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    let help = if long {
        start.render_long_help()
    } else {
        start.render_help()
    };
    help.to_string()
}

#[test]
fn the_long_help_of_start_shows_how_a_harness_is_started() {
    assert!(help_of_start(true).contains("    sh -c <COMMAND>\n"));
}

#[test]
fn the_long_help_of_start_names_what_a_harness_is_told() {
    let help = help_of_start(true);
    let told = [
        "  /harness             ",
        "  /work                ",
        "  CLADENC_HARNESS      /harness\n",
        "  CLADENC_INSTRUCTION  ",
    ];
    assert!(told.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_names_the_variables_a_harness_is_told_the_model_in() {
    let help = help_of_start(true);
    let told = [
        "  CLADENC_MODEL        the name of the model, the value of --model\n",
        "  CLADENC_MODEL_URL    the address the model is served at",
        "  CLADENC_MODEL_KEY    the key the model is called with",
    ];
    assert!(told.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_how_a_harness_reaches_the_machine_cladenc_runs_on() {
    assert!(
        help_of_start(true).contains("host.docker.internal names the machine cladenc\nruns on")
    );
}

#[test]
fn the_long_help_of_start_says_that_a_build_is_not_told_the_model() {
    assert!(help_of_start(true).contains(
        "A build calls no model, and is\ngiven none of the three variables of the model."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_evaluator_is_told_the_model_too() {
    assert!(help_of_start(true).contains(
        "It is\ngiven CLADENC_MODEL, CLADENC_MODEL_URL and CLADENC_MODEL_KEY in its\nenvironment"
    ));
}

#[test]
fn the_short_help_of_start_names_the_flag_that_names_the_file_of_the_key() {
    assert!(help_of_start(false).contains("--model-key-file <FILE>"));
}

#[test]
fn the_long_help_of_start_shows_how_an_image_is_built_from_a_dockerfile() {
    assert!(help_of_start(true).contains("    docker build --tag <NAME> --file <FILE> <DIR>\n"));
}

#[test]
fn the_long_help_of_start_says_what_docker_is_given_to_build_an_image_from() {
    let help = help_of_start(true);
    let given = [
        "  <NAME>  the name cladenc gives the image: ",
        "  <FILE>  a copy of the Dockerfile, kept in image in the run directory\n",
        "  <DIR>   an empty directory, so the Dockerfile has no file to copy into",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_cladenc_tells_docker_to_fetch_nothing() {
    assert!(help_of_start(true).contains("cladenc tells Docker to\nfetch nothing"));
}

#[test]
fn the_long_help_of_start_says_that_resume_builds_no_image() {
    assert!(
        help_of_start(true)
            .contains("`cladenc resume` starts its containers from it and builds nothing.\n")
    );
}

#[test]
fn the_short_help_of_start_names_the_two_flags_of_a_dockerfile() {
    let help = help_of_start(false);
    let flags = ["--dockerfile <FILE>", "--docker-build-timeout <SECONDS>"];
    assert!(flags.iter().all(|flag| help.contains(flag)));
}

#[test]
fn the_long_help_of_start_says_what_each_way_a_harness_ends_means() {
    let help = help_of_start(true);
    let endings = [
        "  exit status 0     the procreation created a harness\n",
        "  exit status 125   Docker did not start the container: the search stops",
        "  any other status  the procreation failed\n",
        "  the time limit    the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_long_help_of_start_says_where_a_harness_is_built() {
    let help = help_of_start(true);
    let said = [
        "Where --harness-build-command is given, a harness is built before it is\n",
        "with /harness as the working directory",
        "builds in the run directory",
    ];
    assert!(said.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_what_each_way_a_build_ends_means() {
    let help = help_of_start(true);
    let endings = [
        "  exit status 0     the harness is built\n",
        "  exit status 125   Docker did not start the container: the search stops",
        "  any other status  a child is recorded as build_failed",
        "  the time limit    the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_short_help_of_start_names_the_option_that_builds_a_harness() {
    assert!(help_of_start(false).contains("--harness-build-command <COMMAND>"));
}

#[test]
fn the_long_help_of_start_shows_how_the_evaluator_is_called() {
    assert!(help_of_start(true).contains("    EVALUATOR <harness> <task> <output>\n"));
}

#[test]
fn the_long_help_of_start_says_what_each_way_the_evaluator_ends_means() {
    let help = help_of_start(true);
    let endings = [
        "  exit status 0   the evaluation succeeded\n",
        "  exit status 1   the evaluation failed\n",
        "  the time limit  the evaluator's process group is killed",
        "  anything else   the search stops",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_short_help_of_start_names_the_flag_that_names_the_benchmark() {
    assert!(help_of_start(false).contains("--benchmark <DIR>"));
}

#[test]
fn the_long_help_of_start_says_that_a_benchmark_holds_a_directory_for_each_task() {
    assert!(
        help_of_start(true)
            .contains("A benchmark is a directory that holds one directory for each task.")
    );
}

#[test]
fn the_long_help_of_start_names_what_the_directory_of_a_task_holds() {
    let help = help_of_start(true);
    let held = [
        "  statement  a file of text that says what is to be done\n",
        "  files      a directory of the files the work starts from",
        "  check      an executable file, the program that says by its exit status\n",
    ];
    assert!(held.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_the_tasks_are_copied_into_the_run_directory() {
    assert!(help_of_start(true).contains("are copied into benchmark in the run directory when a"));
}

#[test]
fn the_long_help_of_start_says_that_the_id_of_a_task_is_the_name_of_its_directory() {
    assert!(help_of_start(true).contains(
        "  <task>     the id of the task, the name of its directory in the\n             benchmark\n"
    ));
}

#[test]
fn the_short_help_of_start_does_not_show_how_a_harness_is_started() {
    assert!(!help_of_start(false).contains("sh -c <COMMAND>"));
}

#[test]
fn the_help_of_every_sub_command_lists_its_flags_in_alphabetical_order() {
    let listed: Vec<Vec<String>> = Cli::command()
        .get_subcommands()
        .map(|sub_command| {
            sub_command
                .get_arguments()
                .map(|flag| flag.get_long().unwrap_or_default().to_owned())
                .collect()
        })
        .collect();
    let mut sorted = listed.clone();
    sorted.iter_mut().for_each(|flags| flags.sort());
    assert_eq!(listed, sorted);
}

#[test]
fn the_help_of_cladenc_names_start_as_the_sub_command_that_starts_a_search() {
    assert!(
        Cli::command()
            .render_help()
            .to_string()
            .contains("  start   Start a new search")
    );
}

#[test]
fn the_help_of_cladenc_lists_no_sub_command_named_help() {
    assert!(
        !Cli::command()
            .render_help()
            .to_string()
            .contains("\n  help ")
    );
}

#[test]
fn help_written_as_a_sub_command_is_refused() {
    assert_eq!(
        Cli::try_parse_from(["cladenc", "help"]).unwrap_err().kind(),
        ErrorKind::InvalidSubcommand
    );
}
