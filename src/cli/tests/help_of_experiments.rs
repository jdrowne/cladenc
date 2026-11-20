use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of_start() -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    start.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_what_experiment_cladenc_conducts_and_at_which_procreations() {
    assert!(long_help_of_start().contains(
        "At each procreation of a harness other than the primogenitor, cladenc\nconducts an \
         experiment to test whether a progenitor is better at\nprocreating better harnesses \
         than the primogenitor."
    ));
}

#[test]
fn the_long_help_of_start_says_how_cladenc_establishes_the_baseline_of_an_experiment() {
    assert!(long_help_of_start().contains(
        "To conduct the\nexperiment, cladenc establishes a baseline: it has the primogenitor \
         create\na harness from the same source code that the progenitor uses to \
         procreate\nthe progenitor's child."
    ));
}

#[test]
fn the_long_help_of_start_names_the_singleton_control_group_harness() {
    assert!(long_help_of_start().contains(
        "The harness the primogenitor creates from the\nsource code of the progenitor is the \
         singleton control group harness."
    ));
}

#[test]
fn the_long_help_of_start_says_which_source_code_runs_and_which_is_changed_for_a_baseline() {
    let help = long_help_of_start();
    let given = [
        "  /var/lib/cladenc/harness  holds a copy of the source code of the\n",
        "                            primogenitor, which is what runs\n",
        "  /var/lib/cladenc/work     holds a second copy of the source code of the\n",
        "                            progenitor, which is what is changed\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_the_primogenitor_is_given_what_the_progenitor_is_given() {
    assert!(long_help_of_start().contains(
        "The primogenitor is given the instruction the progenitor is given, and what\nthe \
         progenitor is given to read:"
    ));
}

#[test]
fn the_long_help_of_start_says_which_children_of_the_progenitor_the_primogenitor_is_given() {
    assert!(long_help_of_start().contains(
        "under /var/log/cladenc/children the children the progenitor procreated\nbefore the \
         progenitor's child of the experiment."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_singleton_control_group_harness_is_benchmarked_in_full() {
    assert!(long_help_of_start().contains(
        "The singleton control group harness is built and benchmarked in full, as\nthe \
         progenitor's child is, and it takes no other part in the directed\nevolution:"
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_singleton_control_group_harness_takes_no_part_in() {
    assert!(long_help_of_start().contains(
        "it is never chosen to procreate, it is counted in no clade, it\nis not named the best \
         harness, and the container it is created in does not\ncount toward --max-procreations."
    ));
}

#[test]
fn the_long_help_of_start_says_that_no_experiment_is_conducted_where_the_primogenitor_procreates() {
    assert!(long_help_of_start().contains(
        "Where the primogenitor procreates, no\nexperiment is conducted: the progenitor's \
         child and the singleton control\ngroup harness would be created in the same way."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_outputs_directory_keeps_of_a_baseline() {
    assert!(long_help_of_start().contains(
        "baselines in the\noutputs directory keeps, in a directory named after the singleton \
         control\ngroup harness, the standard output and standard error of the \
         primogenitor,\nthe id of the progenitor in the file named progenitor, the id of \
         the\nprogenitor's child in the file named child, and how the creation ended in\nthe \
         file named outcome."
    ));
}

#[test]
fn the_long_help_of_start_says_what_report_and_status_say_of_the_experiments() {
    assert!(long_help_of_start().contains(
        "`cladenc report` and `cladenc status` say of the experiments how many were\nconducted, \
         in how many the progenitor's child was created, in how many the\nsingleton control \
         group harness was created, and in how many both were."
    ));
}

#[test]
fn the_long_help_of_start_says_how_the_harnesses_of_the_experiments_are_told_apart() {
    assert!(long_help_of_start().contains(
        "Over the experiments where both were created, each counts the \
         corroboration\ntasks that only the progenitor's child completed and those that only \
         the\nsingleton control group harness completed, and says how likely luck \
         alone\ngives a lead at least as large where the two are created equally well"
    ));
}
