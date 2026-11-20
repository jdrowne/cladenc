use crate::cli::Cli;
use clap::CommandFactory;

fn help_of_cladenc() -> String {
    Cli::command().render_help().to_string()
}

fn long_help_of(sub_command: &str) -> String {
    let mut cladenc = Cli::command();
    let sub_command = cladenc.find_subcommand_mut(sub_command).unwrap();
    sub_command.render_long_help().to_string()
}

#[test]
fn the_help_of_cladenc_says_that_report_is_for_a_directed_evolution_that_has_ended() {
    assert!(
        help_of_cladenc()
            .contains("\n  report  Print the report of a directed evolution that has ended, ")
    );
}

#[test]
fn the_help_of_cladenc_says_that_status_is_for_a_directed_evolution_that_has_not_ended() {
    assert!(help_of_cladenc().contains(
        "\n  status  Print the state so far of a directed evolution that has not ended, "
    ));
}

#[test]
fn the_help_of_report_names_the_flag_that_prints_a_json_object() {
    assert!(long_help_of("report").contains("--json"));
}

#[test]
fn the_help_of_status_names_the_flag_that_prints_a_json_object() {
    assert!(long_help_of("status").contains("--json"));
}

#[test]
fn the_help_of_status_names_the_flag_that_gives_the_outputs_directory() {
    assert!(long_help_of("status").contains("--outputs-dir <DIR>"));
}

#[test]
fn the_long_help_of_start_says_which_directed_evolution_report_and_status_are_each_for() {
    assert!(long_help_of("start").contains(
        "`cladenc report` is for a directed evolution that has ended, and\n`cladenc status` is \
         for one that has not ended."
    ));
}

#[test]
fn the_long_help_of_start_says_that_report_and_status_each_refuse_the_other_kind() {
    assert!(
        long_help_of("start")
            .contains("Each refuses the other\nkind, and says which of the two to call.")
    );
}

#[test]
fn the_long_help_of_start_says_what_start_prints_when_a_directed_evolution_ends() {
    assert!(long_help_of("start").contains(
        "When a directed evolution ends,\n`cladenc start` prints on standard error what \
         `cladenc report` prints,\nafter the line that says that the directed evolution \
         finished, and prints\non standard output the directory of the best harness and nothing \
         else."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_directed_evolution_that_has_not_ended_has_no_best_harness() {
    assert!(
        long_help_of("start")
            .contains("A directed evolution that has not ended has no best harness.")
    );
}

#[test]
fn the_long_help_of_start_says_which_harness_status_names_in_place_of_the_best_harness() {
    assert!(long_help_of("start").contains(
        "In its place\n`cladenc status` names the harness that is ahead so far: the harness \
         that\nwould be named the best harness if the directed evolution ended now."
    ));
}

#[test]
fn the_long_help_of_start_says_that_what_status_prints_is_the_state_so_far_which_can_change() {
    assert!(long_help_of("start").contains(
        "What\n`cladenc status` prints is the state so far, which can change until \
         the\ndirected evolution ends: another harness can be ahead later, and the lead\nof the \
         harness that is ahead so far can grow or shrink."
    ));
}

#[test]
fn the_long_help_of_start_says_once_that_what_status_prints_can_change() {
    assert_eq!(
        long_help_of("start")
            .matches("is the state so far, which can change")
            .count(),
        1
    );
}

#[test]
fn the_long_help_of_start_says_which_three_things_status_says_of_the_harness_ahead_so_far() {
    assert!(long_help_of("start").contains(
        "`cladenc status` says the same three things of the harness that is ahead so\nfar and \
         the primogenitor."
    ));
}

#[test]
fn the_long_help_of_start_says_that_one_of_many_likelihoods_is_low_by_luck_more_often() {
    assert!(long_help_of("start").contains(
        "A person who calls `cladenc status` many times\nreads many likelihoods, and one of \
         many likelihoods is low by luck more\noften than one that is read once."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_likelihood_status_prints_is_not_the_result() {
    assert!(long_help_of("start").contains(
        "Each of them is the state so far, and not\nthe result of the directed evolution."
    ));
}

#[test]
fn the_long_help_of_resume_says_what_resume_prints_when_the_directed_evolution_ends() {
    assert!(long_help_of("resume").contains(
        "When the directed evolution that it continued ends, `cladenc resume` prints\non \
         standard error what `cladenc report` prints, after the line that says\nthat the \
         directed evolution finished, and prints on standard output the\ndirectory of the best \
         harness and nothing else."
    ));
}

#[test]
fn the_long_help_of_resume_says_what_resume_prints_of_a_directed_evolution_that_had_ended() {
    assert!(long_help_of("resume").contains(
        "Of a directed evolution\nthat had ended already, it prints the same after a line that \
         says that the\ndirected evolution had finished."
    ));
}
