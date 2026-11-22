use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of_start() -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    start.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_names_the_artifacts_among_what_cladenc_obeys() {
    assert!(long_help_of_start().contains(
        "  artifacts                       paths in the container that are kept\n                                  \
         with the attempt once the harness has\n                                  ended\n"
    ));
}

#[test]
fn the_long_help_of_start_says_when_the_paths_a_task_lists_under_artifacts_are_kept() {
    assert!(long_help_of_start().contains(
        "Where the task.toml of a task lists paths under artifacts, cladenc keeps\neach with \
         the attempt, a file or a directory with all it holds, as the\nharness left it: it \
         copies them out of the container once the harness has\nended or was stopped, before \
         the tests directory is copied in."
    ));
}

#[test]
fn the_long_help_of_start_says_how_an_entry_under_artifacts_is_written() {
    assert!(long_help_of_start().contains(
        "An entry of\nthe list is a path in the container, or a table that names the path \
         as\nsource."
    ));
}

#[test]
fn the_long_help_of_start_says_where_a_path_listed_under_artifacts_is_kept() {
    assert!(long_help_of_start().contains(
        "The path is kept in artifacts in the directory of the attempt, at\nthe place the \
         table gives as destination, and at the path itself without its\nleading / where the \
         table gives none or the entry is a path alone:\n/app/out/report.json is kept as \
         artifacts/app/out/report.json."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_symbolic_link_at_a_path_under_artifacts_is_followed() {
    assert!(long_help_of_start().contains("A symbolic\nlink at the path is followed."));
}

#[test]
fn the_long_help_of_start_says_what_a_path_that_is_not_in_the_container_keeps() {
    assert!(
        long_help_of_start().contains(
            "A path that is not in the container keeps\nnothing, and the attempt goes on."
        )
    );
}

#[test]
fn the_long_help_of_start_says_which_of_two_entries_kept_at_one_place_is_kept() {
    assert!(long_help_of_start().contains(
        "Where two entries would be kept at the\nsame place, or one inside the other, the \
         first is kept and the other is not."
    ));
}

#[test]
fn the_long_help_of_start_says_what_is_left_out_of_a_directory_that_is_kept() {
    assert!(long_help_of_start().contains(
        "Of a directory, what the table lists as exclude is left out. Each is a\npattern as \
         `tar --exclude` takes one: it leaves out a file or a directory\nwhose name it \
         matches, with all the directory holds, and a pattern with a /\nin it is matched \
         against the path of the file from the directory that is\nkept, or from any directory \
         inside it, so that logs/run.txt leaves out\nrun.txt in every directory named logs."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_characters_of_a_pattern_stand_for() {
    assert!(long_help_of_start().contains(
        "In a pattern, * stands for any run of\ncharacters, / among them, ? for any one \
         character, the characters between [\nand ] for any one of them, and a character \
         after \\ for itself."
    ));
}

#[test]
fn the_long_help_of_start_says_which_entries_under_artifacts_are_refused() {
    assert!(long_help_of_start().contains(
        "Of an entry under artifacts, the path is\nrefused where it holds .., and destination \
         where it begins with /, holds ..,\nor names no file or directory."
    ));
}

#[test]
fn the_long_help_of_start_says_where_what_was_left_at_the_paths_under_artifacts_is_kept() {
    assert!(long_help_of_start().contains(
        "  artifacts      what the harness left at the paths the task lists under\n                 \
         artifacts, where it lists any\n"
    ));
}
