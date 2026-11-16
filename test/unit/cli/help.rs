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

fn help_of_resume(long: bool) -> String {
    let mut cladenc = Cli::command();
    let resume = cladenc.find_subcommand_mut("resume").unwrap();
    let help = if long {
        resume.render_long_help()
    } else {
        resume.render_help()
    };
    help.to_string()
}

#[test]
fn the_long_help_of_resume_says_that_a_cladenc_killed_outright_ends_none_of_its_containers() {
    assert!(help_of_resume(true).contains(
        "A cladenc that is killed outright, as by `kill -9`, ends none of its\ncontainers, and \
         each goes on until its time limit has passed."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_it_first_removes_the_containers_the_search_left_running() {
    assert!(help_of_resume(true).contains(
        "Before it\ndoes anything else, `cladenc resume` removes the containers that the \
         search\nleft running, so that it never works beside one of them."
    ));
}

#[test]
fn the_long_help_of_resume_shows_the_label_that_every_container_of_a_search_carries() {
    assert!(help_of_resume(true).contains(
        "\n    cladenc.search=<DIR>\n\nwhere <DIR> is the path of the run directory, with every \
         symbolic link in\nit resolved."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_the_containers_are_removed_once_no_other_cladenc_is_there() {
    assert!(help_of_resume(true).contains(
        "Once it has the run directory to itself, `cladenc resume` asks\nDocker for the \
         containers that carry the label of its search, removes them,\nand says how many it \
         removed."
    ));
}

#[test]
fn the_long_help_of_resume_says_what_it_does_where_docker_cannot_be_asked() {
    assert!(
        help_of_resume(true).contains(
            "Where Docker cannot be asked, it exits with\nstatus 2 and continues nothing."
        )
    );
}

#[test]
fn the_long_help_of_resume_shows_how_the_containers_of_a_search_are_listed_by_hand() {
    assert!(help_of_resume(true).contains(
        "Where the search is not resumed, its containers are removed by hand.\n\n    docker ps \
         --filter label=cladenc.search=<DIR>\n\nlists them"
    ));
}

#[test]
fn the_long_help_of_resume_says_how_a_container_is_ended_by_hand() {
    assert!(help_of_resume(true).contains(
        "their names begin cladenc-, and `docker kill` with a name ends\nthat container. Docker \
         removes a container of cladenc once it has ended."
    ));
}

#[test]
fn the_short_help_of_resume_does_not_show_the_label_of_a_search() {
    assert!(!help_of_resume(false).contains("cladenc.search"));
}

#[test]
fn the_long_help_of_start_says_that_resume_removes_the_containers_a_search_left_running() {
    assert!(help_of_start(true).contains(
        "Until then a container that was left goes on. `cladenc resume` removes the\ncontainers \
         that the search left running before it does anything else, and\nits long help says \
         how they are found, and how they are removed by hand\nwhere the search is not resumed."
    ));
}

#[test]
fn the_long_help_of_start_shows_how_a_harness_is_started() {
    assert!(help_of_start(true).contains("    sh -c <TIMER> <COMMAND> <SECONDS>\n"));
}

#[test]
fn the_long_help_of_start_shows_the_timer_that_every_container_is_started_under() {
    assert!(help_of_start(true).contains(
        "  <TIMER>    sh -c \"$0\" & run=$!; (sleep \"$1\"; kill -9 $run) & wait $run\n"
    ));
}

#[test]
fn the_long_help_of_start_says_which_command_the_timer_of_a_harness_starts() {
    assert!(help_of_start(true).contains("  <COMMAND>  the value of --harness-run-command\n"));
}

#[test]
fn the_long_help_of_start_says_how_long_the_timer_of_a_container_waits() {
    assert!(help_of_start(true).contains(
        "  <SECONDS>  what is left of the time limit when the container is started,\n             \
         in whole seconds, and five seconds more\n"
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_timer_does() {
    assert!(help_of_start(true).contains(
        "The timer is a line of shell that starts the command, ends it once the\nseconds have \
         passed, and hands on its exit status."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_container_ends_itself_when_cladenc_was_killed_outright() {
    assert!(help_of_start(true).contains(
        "a container ends itself once its time\nlimit has passed, also when cladenc was killed \
         outright, as by `kill -9`,\nand is not there to remove it."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_image_has_to_hold_for_the_timer() {
    assert!(help_of_start(true).contains("The image of the search has to hold sh and\nsleep.\n"));
}

#[test]
fn the_long_help_of_start_shows_how_a_harness_is_built() {
    assert!(help_of_start(true).contains(
        "in a container of the same image, as\n\n    sh -c <TIMER> <COMMAND> <SECONDS>\n\nwhere \
         <COMMAND> is the value of that flag and the timer is the same one,\n"
    ));
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
fn the_short_help_of_start_names_the_flag_that_limits_the_containers_running_at_once() {
    assert!(help_of_start(false).contains("--max-concurrent-containers <COUNT>"));
}

#[test]
fn the_short_help_of_start_says_that_the_limit_is_the_most_containers_running_at_once() {
    assert!(help_of_start(false).contains("The most containers running at once"));
}

#[test]
fn the_long_help_of_start_shows_how_an_image_is_built_from_a_dockerfile() {
    assert!(
        help_of_start(true).contains(
            "    timeout -s KILL <SECONDS> docker build --tag <NAME> --file <FILE> <DIR>\n"
        )
    );
}

#[test]
fn the_long_help_of_start_says_what_docker_is_given_to_build_an_image_from() {
    let help = help_of_start(true);
    let given = [
        "  <NAME>     the name cladenc gives the image: ",
        "  <FILE>     a copy of the Dockerfile, kept in image in the run directory\n",
        "  <DIR>      an empty directory, so the Dockerfile has no file to copy into",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_how_long_the_timeout_of_the_build_of_the_image_waits() {
    assert!(
        help_of_start(true)
            .contains("  <SECONDS>  the value of --docker-build-timeout, and five seconds more\n")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_build_of_the_image_ends_itself_at_its_time_limit() {
    assert!(help_of_start(true).contains(
        "cladenc ends a build that\nruns past its time limit, and the build ends itself five \
         seconds later\nwhere cladenc was killed outright, as by `kill -9`, and is not there to \
         end\nit."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_machine_has_to_hold_timeout() {
    assert!(help_of_start(true).contains(
        "The machine cladenc runs on has to hold timeout, which the package\ncoreutils brings."
    ));
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
            .contains("`cladenc resume` starts its containers from\nit and builds nothing.\n")
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
fn the_long_help_of_start_says_what_the_harness_of_an_assessment_is_given_of_its_task() {
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
    assert!(help_of_start(true).contains("\n    sh -c <TIMER> /task/check <SECONDS>\n"));
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
fn the_long_help_of_start_says_what_each_way_an_assessment_ends_means() {
    let help = help_of_start(true);
    let endings = [
        "  the check exits with status 0    the assessment succeeded\n",
        "  the check exits with another     the assessment failed\n",
        "  exit status 125 of either step   Docker did not start the container",
        "  the time limit                   the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_long_help_of_start_names_what_the_run_directory_keeps_of_an_assessment() {
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
fn the_long_help_of_start_says_where_the_directory_of_an_assessment_is() {
    assert!(help_of_start(true).contains(
        "in a directory\nnamed after the harness, in it one named after the benchmark, and in \
         that\none named after the task, which holds"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_procreation_finds_its_assessments_by_benchmark() {
    assert!(help_of_start(true).contains(
        "one\ndirectory for each benchmark, and in it one for each task, holding what \
         the\nassessment of that task kept."
    ));
}

#[test]
fn the_short_help_of_start_does_not_show_how_a_harness_is_started() {
    assert!(!help_of_start(false).contains("sh -c <TIMER>"));
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
