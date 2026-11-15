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
fn the_long_help_of_run_shows_how_the_mutator_is_called() {
    assert!(help_of_run(true).contains("    MUTATOR <parent> <child> <evaluations> <output>\n"));
}

#[test]
fn the_long_help_of_run_shows_how_the_evaluator_is_called() {
    assert!(help_of_run(true).contains("    EVALUATOR <harness> <task> <output>\n"));
}

#[test]
fn the_long_help_of_run_says_what_each_way_of_ending_means() {
    let help = help_of_run(true);
    let endings = [
        "  exit status 0   the mutation created a harness; the evaluation succeeded\n",
        "  exit status 1   the mutation failed; the evaluation failed\n",
        "  the time limit  the program's process group is killed",
        "  anything else   the search stops",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_short_help_of_run_does_not_show_how_the_programs_are_called() {
    assert!(!help_of_run(false).contains("MUTATOR <parent>"));
}
