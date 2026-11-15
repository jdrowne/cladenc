use cladenc::cli::Cli;
use clap::CommandFactory;

fn help_of_run(long: bool) -> String {
    let mut cladenc = Cli::command();
    let run = cladenc.find_subcommand_mut("run").unwrap();
    let help = if long {
        run.render_long_help()
    } else {
        run.render_help()
    };
    help.to_string()
}

#[test]
fn the_long_help_of_run_shows_how_a_harness_is_started() {
    assert!(help_of_run(true).contains("    sh -c <COMMAND>\n"));
}

#[test]
fn the_long_help_of_run_names_what_a_harness_is_told() {
    let help = help_of_run(true);
    let told = [
        "  /harness             ",
        "  /work                ",
        "  CLADENC_HARNESS      /harness\n",
        "  CLADENC_INSTRUCTION  ",
    ];
    assert!(told.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_run_says_what_each_way_a_harness_ends_means() {
    let help = help_of_run(true);
    let endings = [
        "  exit status 0     the procreation created a harness\n",
        "  exit status 125   Docker did not start the container: the search stops",
        "  any other status  the procreation failed\n",
        "  the time limit    the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_long_help_of_run_shows_how_the_evaluator_is_called() {
    assert!(help_of_run(true).contains("    EVALUATOR <harness> <task> <output>\n"));
}

#[test]
fn the_long_help_of_run_says_what_each_way_the_evaluator_ends_means() {
    let help = help_of_run(true);
    let endings = [
        "  exit status 0   the evaluation succeeded\n",
        "  exit status 1   the evaluation failed\n",
        "  the time limit  the evaluator's process group is killed",
        "  anything else   the search stops",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_short_help_of_run_does_not_show_how_a_harness_is_started() {
    assert!(!help_of_run(false).contains("sh -c <COMMAND>"));
}
