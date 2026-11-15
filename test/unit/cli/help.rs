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
fn the_long_help_of_start_says_that_a_check_is_not_told_the_model() {
    assert!(help_of_start(true).contains(
        "A check calls no model, and is given none of the three variables of the\nmodel."
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
fn the_long_help_of_start_says_that_cladenc_fetches_no_image_and_has_no_flag_for_a_registry() {
    assert!(help_of_start(true).contains(
        "cladenc fetches no image, and has no flag for a registry, the server that\nDocker \
         fetches an image from."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_name_of_an_image_says_which_registry_it_is_of() {
    assert!(help_of_start(true).contains(
        "The name of an image says which registry it\nis of, as \
         registry.example:5000/tools/harness:1 does, and a name that says\nnone is of Docker Hub."
    ));
}

#[test]
fn the_long_help_of_start_says_how_an_image_is_brought_to_the_machine() {
    assert!(help_of_start(true).contains(
        "The image that --docker-image names has to be on\nthe machine before the search: \
         fetched there with `docker pull`, carried\nthere as a file with `docker save` and \
         `docker load`, or built there."
    ));
}

#[test]
fn the_long_help_of_start_says_that_docker_is_told_never_to_fetch_the_image_of_a_container() {
    assert!(help_of_start(true).contains(
        "cladenc tells Docker never to fetch the image of a container, so a search\nwhose image \
         is not on the machine stops at its first container, and\n`cladenc resume` continues \
         it once the image is there."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_dockerfile_can_be_built_with_no_connection() {
    assert!(help_of_start(true).contains(
        "A Dockerfile whose\nbase image is on the machine, and whose steps fetch nothing, is \
         built with\nno connection to the internet."
    ));
}

#[test]
fn the_long_help_of_start_says_that_docker_fetches_a_base_image_the_machine_does_not_hold() {
    assert!(help_of_start(true).contains(
        "cladenc does not ask Docker to fetch a base\nimage and cannot forbid it: Docker \
         fetches a base image that is not on the\nmachine, from the registry of its name."
    ));
}

#[test]
fn the_long_help_of_start_says_that_resume_builds_no_image() {
    assert!(
        help_of_start(true)
            .contains("`cladenc resume`\nstarts its containers from it and builds nothing.\n")
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
fn the_long_help_of_start_says_what_the_harness_of_an_evaluation_is_given_of_its_task() {
    assert!(help_of_start(true).contains(
        "/work holds a copy of the files of the task, and CLADENC_INSTRUCTION holds\nthe \
         statement of the task."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_harness_can_neither_read_nor_change_the_check() {
    assert!(help_of_start(true).contains("so it can neither read nor change the check."));
}

#[test]
fn the_long_help_of_start_shows_how_the_check_of_a_task_is_called() {
    assert!(help_of_start(true).contains("\n    /task/check\n"));
}

#[test]
fn the_long_help_of_start_names_what_a_check_is_given() {
    let help = help_of_start(true);
    let given = [
        "  /work         holds what the harness left, and is the working directory\n",
        "  /task         holds the directory of the task, to read and not to change\n",
        "  CLADENC_TASK  /task\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_what_each_way_an_evaluation_ends_means() {
    let help = help_of_start(true);
    let endings = [
        "  the check exits with status 0    the evaluation succeeded\n",
        "  the check exits with another     the evaluation failed\n",
        "  exit status 125 of either step   Docker did not start the container",
        "  the time limit                   the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_long_help_of_start_names_what_the_run_directory_keeps_of_an_evaluation() {
    let help = help_of_start(true);
    let kept = [
        "  outcome    one word: success, failure or timed_out\n",
        "  statement  the statement of the task\n",
        "  work       the files of the task as the harness left them\n",
        "  stdout     the standard output of the harness\n",
        "  stderr     the standard error of the harness\n",
        "  check      the standard output and the standard error of the check",
    ];
    assert!(kept.iter().all(|line| help.contains(line)));
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
    assert!(help_of_start(true).contains(
        "The tasks are copied into\nbenchmarks in the run directory when a search is started, each \
         into a\ndirectory named after its benchmark"
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_id_of_a_task_is_the_name_of_its_directory() {
    assert!(help_of_start(true).contains(
        "The name\nof a task's directory is the id of the task, made of letters, digits, dots,\n\
         underscores and hyphens"
    ));
}

#[test]
fn the_short_help_of_start_says_that_the_benchmark_flag_is_given_once_for_each_benchmark() {
    assert!(help_of_start(false).contains("given once for each benchmark"));
}

#[test]
fn the_long_help_of_start_says_what_a_search_given_several_benchmarks_looks_for() {
    assert!(help_of_start(true).contains(
        "A search that\nis given several tries its harnesses on the tasks of all of them, and \
         looks\nfor the one harness that passes the largest share of those tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_that_every_task_counts_the_same() {
    assert!(
        help_of_start(true)
            .contains("Every\ntask counts the same, so a benchmark with more tasks weighs more.")
    );
}

#[test]
fn the_long_help_of_start_says_how_a_task_of_a_search_is_known() {
    assert!(help_of_start(true).contains(
        "a task of a search is known by the two together,\nas\n\n    <BENCHMARK>/<TASK>\n\nso \
         two benchmarks may each hold a task of the same id."
    ));
}

#[test]
fn the_long_help_of_start_says_that_two_benchmarks_of_the_same_name_are_refused() {
    assert!(
        help_of_start(true)
            .contains("Two benchmarks whose\ndirectories have the same name are refused.")
    );
}

#[test]
fn the_long_help_of_start_says_where_the_directory_of_an_evaluation_is() {
    assert!(help_of_start(true).contains(
        "in a directory\nnamed after the harness, in it one named after the benchmark, and in \
         that\none named after the task, which holds"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_procreation_finds_its_evaluations_by_benchmark() {
    assert!(help_of_start(true).contains(
        "one\ndirectory for each benchmark, and in it one for each task, holding what \
         the\nevaluation of that task kept."
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
