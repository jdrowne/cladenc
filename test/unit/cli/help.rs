use cladenc::cli::Cli;
use clap::CommandFactory;

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
