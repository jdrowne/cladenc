use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_names_the_artifacts_among_what_cladenc_obeys() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "artifacts paths in the sandbox that are kept with the trial once the harness has ended"
    ));
}

#[test]
fn the_manual_of_start_says_when_the_paths_a_task_lists_under_artifacts_are_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the task.toml of a task lists paths under artifacts, cladenc keeps each with the \
         trial, a file or a directory with all it holds, as the harness left it: it copies them \
         out of the sandbox once the harness has ended or was stopped, before the tests \
         directory is copied in."
    ));
}

#[test]
fn the_manual_of_start_says_how_an_entry_under_artifacts_is_written() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "An entry of the list is a path in the sandbox, or a table that names the path as source."
    ));
}

#[test]
fn the_manual_of_start_says_where_a_path_listed_under_artifacts_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The path is kept in artifacts in the directory of the trial, at the place the table \
         gives as destination, and at the path itself without its leading / where the table gives \
         none or the entry is a path alone: /app/out/report.json is kept as \
         artifacts/app/out/report.json."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_symbolic_link_at_a_path_under_artifacts_is_followed() {
    assert!(
        manual_on_one_line("cladenc-start").contains("A symbolic link at the path is followed.")
    );
}

#[test]
fn the_manual_of_start_says_what_a_path_that_is_not_in_the_sandbox_keeps() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("A path that is not in the sandbox keeps nothing, and the trial goes on.")
    );
}

#[test]
fn the_manual_of_start_says_which_of_two_entries_kept_at_one_place_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where two entries would be kept at the same place, or one inside the other, the first is \
         kept and the other is not."
    ));
}

#[test]
fn the_manual_of_start_says_what_is_left_out_of_a_directory_that_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Of a directory, what the table lists as exclude is left out. Each is a pattern as tar \
         --exclude takes one: it leaves out a file or a directory whose name it matches, with all \
         the directory holds, and a pattern with a / in it is matched against the path of the file \
         from the directory that is kept, or from any directory inside it, so that logs/run.txt \
         leaves out run.txt in every directory named logs."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_characters_of_a_pattern_stand_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "In a pattern, * stands for any run of characters, / among them, ? for any one character, \
         the characters between [ and ] for any one of them, and a character after \\ for itself."
    ));
}

#[test]
fn the_manual_of_start_says_which_entries_under_artifacts_are_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Of an entry under artifacts, the path is refused where it holds .., and destination where \
         it begins with /, holds .., or names no file or directory."
    ));
}

#[test]
fn the_manual_of_start_says_where_what_was_left_at_the_paths_under_artifacts_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "artifacts what the harness left at the paths the task lists under artifacts, where it \
         lists any, and in /logs/artifacts"
    ));
}

#[test]
fn the_manual_of_start_says_how_cladenc_copies_out_of_a_sandbox() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "What cladenc copies out of a sandbox, it has Docker print as an archive and unpacks \
         itself: what the working directory of the harness holds, and what /logs/verifier holds, \
         as docker cp <SANDBOX>:<DIR>/. - and a path that is kept as ARTIFACTS says, as docker cp \
         --follow-link <SANDBOX>:<PATH> - where <SANDBOX> is the name that cladenc gave the \
         sandbox."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_directory_that_cannot_be_written_in_is_copied_out_whole() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A directory that cannot be written in is copied out with all it holds, and the copy is \
         a directory that its owner can write in."
    ));
}

#[test]
fn the_manual_of_start_says_which_files_are_not_copied_out_of_a_sandbox() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A file that is not a regular file, a directory or a symbolic link, such as a named pipe \
         or a device, is not copied out, and neither is another name of such a file."
    ));
}
