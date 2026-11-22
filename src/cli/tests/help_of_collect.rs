use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of(sub_command: &str) -> String {
    let mut cladenc = Cli::command();
    let sub_command = cladenc.find_subcommand_mut(sub_command).unwrap();
    sub_command.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_that_the_container_of_an_attempt_holds_logs_artifacts() {
    assert!(long_help_of("start").contains(
        "The container of an attempt holds /logs/artifacts when the harness is\nstarted: an \
         empty directory that belongs to the user the harness runs as, as\nin Harbor."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_a_harness_left_in_logs_artifacts_is_kept() {
    assert!(long_help_of("start").contains(
        "What the harness left in it is kept as a path that the task lists\nis, as \
         artifacts/logs/artifacts, though the task lists no such path, and\nnothing is kept \
         where the harness left nothing there."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_task_that_lists_logs_artifacts_itself_gets() {
    assert!(long_help_of("start").contains(
        "A task that lists\n/logs/artifacts itself, or a path that is kept at the same place, \
         has that\nentry obeyed in its place."
    ));
}

#[test]
fn the_long_help_of_start_says_when_the_commands_under_collect_are_run() {
    assert!(long_help_of("start").contains(
        "Where the task.toml of a task lists commands under verifier.collect, cladenc\nruns \
         each in the container of the harness once the harness has ended or was\nstopped, \
         before it keeps any path, in the order the task lists them."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_command_under_collect_is_run() {
    assert!(long_help_of("start").contains(
        "A\ncommand is run as the harness is, under the same timer and in the \
         working\ndirectory of the environment, as the user its entry names as user, or \
         the\nuser of the environment where it names none."
    ));
}

#[test]
fn the_long_help_of_start_says_what_time_limit_a_command_under_collect_has() {
    assert!(long_help_of("start").contains(
        "It has the time limit that its\nentry gives as timeout_sec, or 60 seconds where it \
         gives none."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_command_under_collect_that_fails_ends_no_attempt() {
    assert!(long_help_of("start").contains(
        "A command\nthat exits with another status than 0, or runs past its time limit, does \
         not\nend the attempt: the next command is run, and the paths are kept as they\nthen \
         are."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_a_command_under_collect_printed_is_kept() {
    assert!(long_help_of("start").contains(
        "collect in the directory of the attempt keeps what each command\nprinted, in a \
         directory named after the place of the command in the list,\ncounted from 1."
    ));
}

#[test]
fn the_long_help_of_start_names_the_commands_under_collect_among_what_cladenc_obeys() {
    assert!(long_help_of("start").contains(
        "  collect under verifier          commands that are run in the \
         container\n                                  once the harness has ended, before \
         the\n                                  paths under artifacts are kept\n"
    ));
}

#[test]
fn the_long_help_of_start_names_collect_among_what_an_attempt_keeps() {
    assert!(long_help_of("start").contains(
        "  collect        what each command under verifier.collect printed, in \
         a\n                 directory named after its place in the list, where \
         the\n                 task lists any\n"
    ));
}
