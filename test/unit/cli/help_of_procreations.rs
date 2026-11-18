use cladenc::cli::Cli;
use clap::CommandFactory;

fn long_help_of_start() -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    start.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_that_a_procreation_finds_its_attempts_by_benchmark() {
    assert!(long_help_of_start().contains(
        "one directory\nfor each benchmark, and in it one for each task, holding what the \
         attempt\nof that task kept."
    ));
}

#[test]
fn the_long_help_of_start_says_where_a_procreation_finds_its_attempts() {
    assert!(long_help_of_start().contains(
        "/var/log/cladenc/attempts holds the finished attempts\nof the harness at the tasks \
         that are not held out, to read:"
    ));
}

#[test]
fn the_long_help_of_start_says_of_which_harnesses_a_harness_that_procreates_is_given_more() {
    assert!(long_help_of_start().contains(
        "A harness that procreates is given more to read, and nothing more to\nchange: what \
         was kept of each harness it descends from, back to the\nprimogenitor, and of each \
         harness it made in an earlier procreation that\nhas ended, whether that one was \
         created or not. It is given nothing of\nany other harness."
    ));
}

#[test]
fn the_long_help_of_start_names_where_the_line_and_the_children_of_a_harness_are() {
    let help = long_help_of_start();
    let given = [
        "  /var/log/cladenc/line/<ID>      a harness it descends from\n",
        "  /var/log/cladenc/children/<ID>  a harness it made before\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_names_what_is_given_of_a_harness_of_the_line_or_a_child() {
    let help = long_help_of_start();
    let given = [
        "  harness      the source code of that harness\n",
        "  attempts     its attempts at the tasks that are not held out, laid out\n",
        "  procreation  what the harness that made it printed, in stdout and\n",
        "               progenitor and one word in the file named outcome:\n",
        "               created, failed, build_failed or timed_out\n",
        "  build        what its build printed, in stdout and stderr\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_which_directory_of_such_a_harness_is_not_there() {
    assert!(long_help_of_start().contains(
        "A directory that would hold nothing is not there: the primogenitor has no\nprocreation, \
         a harness that was not created has no attempts, and a harness\nthat was not built has \
         no build. Nothing of a holdout task is in any of\nthem."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_outputs_directory_keeps_of_a_procreation() {
    assert!(long_help_of_start().contains(
        "procreations in the\noutputs directory keeps, in a directory named after the child, \
         the\nstandard output and standard error of the harness that made it, the id of\nthat \
         harness in the file named progenitor, and how the procreation ended\nin the file \
         named outcome."
    ));
}

#[test]
fn the_long_help_of_start_says_what_kind_of_change_the_instruction_asks_for() {
    assert!(long_help_of_start().contains(
        "The instruction asks the harness that procreates for a change to how its\nchild works \
         on any task: one that answers something it saw in several of\nits attempts, not in \
         one, and that leaves in the source code nothing that\nnames a task, a file of a task \
         or the answer to one."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_counts_taking_out_as_an_improvement() {
    assert!(
        long_help_of_start()
            .contains("It says that taking\nout what does not help is an improvement as well,")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_tells_of_the_child_procreating_in_its_turn() {
    assert!(long_help_of_start().contains(
        "that the child may be\nasked in its turn to improve a copy of its own source code, so \
         that making\nthe child better at that counts,"
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_tells_how_the_child_is_judged() {
    assert!(long_help_of_start().contains(
        "and that the child is judged by tasks\nthe harness that procreates is not shown."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_instruction_is_a_request_cladenc_does_not_enforce() {
    assert!(
        long_help_of_start().contains("An instruction is a request:\ncladenc enforces none of it.")
    );
}

#[test]
fn the_long_help_of_start_says_that_a_harness_procreates_only_once_benchmarked_in_full() {
    assert!(long_help_of_start().contains(
        "A harness procreates only once it has been benchmarked in full: it has\nattempted \
         every task of every benchmark of the directed evolution, so that\nit finds every task \
         that is not held out in /var/log/cladenc/attempts."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_attempt_takes_the_place_of_a_procreation() {
    assert!(long_help_of_start().contains(
        "Where a procreation is due and no harness has been benchmarked in full\nyet, an \
         attempt is started in its place: the primogenitor is benchmarked\nin full before the \
         first procreation."
    ));
}

#[test]
fn the_long_help_of_start_says_that_every_harness_is_benchmarked_in_full() {
    assert!(long_help_of_start().contains(
        "Every harness is benchmarked in full once it is created, whether it\nprocreates or not."
    ));
}

#[test]
fn the_long_help_of_start_says_which_harness_attempts_next() {
    assert!(long_help_of_start().contains(
        "It does not choose which harness attempts next: the created harness\nwith the lowest \
         id that has a task left attempts it."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_most_procreations_counts() {
    assert!(long_help_of_start().contains(
        "--max-procreations gives the most procreations a directed evolution\nstarts. A \
         procreation counts once it is started, whether or not it creates\nits child."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_directed_evolution_ends() {
    assert!(long_help_of_start().contains(
        "The directed evolution ends once that many have been started,\neach of them has ended, \
         and every created harness has been benchmarked in\nfull."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_growth_never_keeps_a_procreation_from_starting() {
    assert!(long_help_of_start().contains(
        "where it lets none start and nothing else is left to start,\na procreation is started \
         all the same."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_best_of_harnesses_that_are_alike_is_ahead_by_luck() {
    assert!(long_help_of_start().contains(
        "The best harness is named among many, each benchmarked once, and a model\ndoes not \
         answer the same way twice. So the best of many harnesses that are\nall alike has \
         still completed more holdout tasks than the primogenitor, by\nluck."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_directed_evolution_in_which_no_harness_changes_is_carried_out()
{
    assert!(long_help_of_start().contains(
        "How many more is measured by a directed evolution in which no\nharness changes: one \
         carried out with the values of the directed evolution\nit is set beside, and with a \
         run command that ends at once with exit\nstatus 0 where the harness is started for a \
         procreation, as"
    ));
}

#[test]
fn the_long_help_of_start_gives_a_run_command_that_changes_nothing_in_a_procreation() {
    assert!(long_help_of_start().contains(
        "\n    --harness-run-command \\\n        '[ -d /var/log/cladenc/attempts ] && exit 0; \
         <COMMAND>'\n\nwhere <COMMAND> is the run command of the other."
    ));
}

#[test]
fn the_long_help_of_start_says_why_that_run_command_changes_nothing() {
    assert!(long_help_of_start().contains(
        "/var/log/cladenc/attempts\nis there for a procreation and not for an attempt, so \
         every procreation\ncreates a child that is the source code of the primogenitor, \
         unchanged,\nand calls no model."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_directed_evolution_in_which_no_harness_changes_holds() {
    assert!(long_help_of_start().contains(
        "Such a directed evolution holds as many children as\n--max-procreations gives, each \
         of them benchmarked in full."
    ));
}

#[test]
fn the_long_help_of_start_says_how_the_report_of_such_a_directed_evolution_is_read() {
    assert!(long_help_of_start().contains(
        "In\n`cladenc report` of it, how far apart the counts of the harnesses lie is\nwhat \
         luck does to the count of one harness, and how many more holdout\ntasks the best \
         harness completed than the primogenitor is what luck and\nthe choosing give \
         together, with nothing changed."
    ));
}

#[test]
fn the_long_help_of_start_says_how_far_a_best_harness_is_ahead_by_a_change() {
    assert!(long_help_of_start().contains(
        "The best harness of the\nother directed evolution is ahead of its primogenitor by a \
         change only as\nfar as it is ahead by more than that."
    ));
}
