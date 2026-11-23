use std::path::{Path, PathBuf};

use crate::containers::artifacts;
use crate::file_system::tests::double::Memory;
use crate::file_system::{FileSystem, Kind};

const FILES: [&str; 6] = [
    "notes.txt",
    "draft.tmp",
    "logs/run.txt",
    "logs/run.tmp",
    "logs/cache/entry",
    "cache/entry",
];

fn tree_of(files: &[&str]) -> (Memory, PathBuf) {
    let directory = Memory::default();
    let tree = directory.path().join("kept");
    directory.create_dir(&tree).unwrap();
    for file in files {
        let file = tree.join(file);
        directory.create_dir_all(file.parent().unwrap()).unwrap();
        directory.write(file, "").unwrap();
    }
    (directory, tree)
}

fn listed(files: &Memory, tree: &Path, below: &str, found: &mut Vec<String>) {
    for name in files.names_in(tree) {
        let path = format!("{below}{name}");
        let entry = tree.join(&name);
        if files.found(&entry).unwrap().kind == Kind::Directory {
            listed(files, &entry, &format!("{path}/"), found);
        }
        found.push(path);
    }
}

fn everything_in(files: &Memory, tree: &Path) -> Vec<String> {
    let mut found = Vec::new();
    listed(files, tree, "", &mut found);
    found.sort();
    found
}

fn patterns(exclude: &[&str]) -> Vec<String> {
    exclude
        .iter()
        .map(|pattern| (*pattern).to_owned())
        .collect()
}

fn left_of(files: &[&str], exclude: &[&str]) -> Vec<String> {
    let (directory, tree) = tree_of(files);
    artifacts::leave_out(&directory, &tree, &patterns(exclude));
    everything_in(&directory, &tree)
}

fn linked_to_a_directory_apart(exclude: &[&str]) -> (Memory, Vec<String>) {
    let (directory, tree) = tree_of(&["notes.txt"]);
    let apart = directory.path().join("apart");
    directory.create_dir(&apart).unwrap();
    directory.write(apart.join("draft.tmp"), "").unwrap();
    directory.symlink(&apart, tree.join("link")).unwrap();
    artifacts::leave_out(&directory, &tree, &patterns(exclude));
    let left = everything_in(&directory, &tree);
    (directory, left)
}

#[test]
fn a_directory_with_no_pattern_keeps_everything() {
    assert_eq!(
        left_of(&["notes.txt", "logs/run.tmp"], &[]),
        ["logs", "logs/run.tmp", "notes.txt"]
    );
}

#[test]
fn a_pattern_leaves_out_the_files_it_matches_by_name_at_any_depth() {
    assert_eq!(
        left_of(&FILES, &["*.tmp"]),
        [
            "cache",
            "cache/entry",
            "logs",
            "logs/cache",
            "logs/cache/entry",
            "logs/run.txt",
            "notes.txt"
        ]
    );
}

#[test]
fn a_pattern_leaves_out_the_directories_it_matches_by_name_with_all_they_hold() {
    assert_eq!(
        left_of(&FILES, &["cache"]),
        [
            "draft.tmp",
            "logs",
            "logs/run.tmp",
            "logs/run.txt",
            "notes.txt"
        ]
    );
}

#[test]
fn a_pattern_without_a_wildcard_matches_no_part_of_a_name() {
    assert_eq!(
        left_of(&["cache", "cached", "my-cache"], &["cache"]),
        ["cached", "my-cache"]
    );
}

#[test]
fn every_pattern_of_several_leaves_out_what_it_matches() {
    assert_eq!(left_of(&FILES, &["*.tmp", "cache", "logs"]), ["notes.txt"]);
}

#[test]
fn a_pattern_with_a_slash_matches_the_end_of_a_path_from_any_directory() {
    assert_eq!(
        left_of(
            &["logs/run.txt", "deep/logs/run.txt", "run.txt"],
            &["logs/run.txt"]
        ),
        ["deep", "deep/logs", "logs", "run.txt"]
    );
}

#[test]
fn a_pattern_that_begins_with_a_dot_and_a_slash_matches_at_the_top_alone() {
    assert_eq!(
        left_of(&["run.txt", "logs/run.txt"], &["./run.txt"]),
        ["logs", "logs/run.txt"]
    );
}

#[test]
fn a_star_matches_a_slash_as_well() {
    assert_eq!(
        left_of(
            &["logs/deep/run.txt", "logs.txt", "notes.txt"],
            &["logs*txt"]
        ),
        ["logs", "logs/deep", "notes.txt"]
    );
}

#[test]
fn a_star_at_the_end_of_a_pattern_matches_the_rest_of_a_name() {
    assert_eq!(
        left_of(&["cache-1", "cache", "notes.txt"], &["cache*"]),
        ["notes.txt"]
    );
}

#[test]
fn a_star_matches_again_further_on_where_what_follows_it_does_not_match_at_first() {
    assert_eq!(
        left_of(&["a.b.tmp.tmp", "a.b.tmp.txt"], &["a*.tmp"]),
        ["a.b.tmp.txt"]
    );
}

#[test]
fn a_question_mark_matches_one_character() {
    assert_eq!(left_of(&["a.txt", "ab.txt"], &["?.txt"]), ["ab.txt"]);
}

#[test]
fn characters_in_brackets_match_one_character_that_is_among_them() {
    assert_eq!(
        left_of(&["a.txt", "b.txt", "c.txt", "d.txt"], &["[abc].txt"]),
        ["d.txt"]
    );
}

#[test]
fn two_characters_in_brackets_with_a_hyphen_between_match_one_character_from_one_to_the_other() {
    assert_eq!(
        left_of(&["a.txt", "c.txt", "d.txt"], &["[a-c].txt"]),
        ["d.txt"]
    );
}

#[test]
fn a_hyphen_at_the_end_of_brackets_matches_a_hyphen() {
    assert_eq!(
        left_of(&["a.txt", "-.txt", "b.txt"], &["[a-].txt"]),
        ["b.txt"]
    );
}

#[test]
fn brackets_that_begin_with_an_exclamation_mark_match_one_character_that_is_not_among_them() {
    assert_eq!(left_of(&["a.txt", "b.txt"], &["[!a].txt"]), ["a.txt"]);
}

#[test]
fn brackets_that_begin_with_a_caret_match_one_character_that_is_not_among_them() {
    assert_eq!(left_of(&["a.txt", "b.txt"], &["[^a].txt"]), ["a.txt"]);
}

#[test]
fn a_closing_bracket_that_comes_first_in_brackets_is_one_of_the_characters() {
    assert_eq!(left_of(&["].txt", "a.txt"], &["[]].txt"]), ["a.txt"]);
}

#[test]
fn an_opening_bracket_that_is_never_closed_matches_itself() {
    assert_eq!(left_of(&["[a.txt", "a.txt"], &["[a.txt"]), ["a.txt"]);
}

#[test]
fn a_backslash_makes_the_character_after_it_match_itself() {
    assert_eq!(left_of(&["*.txt", "a.txt"], &["\\*.txt"]), ["a.txt"]);
}

#[test]
fn a_backslash_at_the_end_of_a_pattern_matches_itself() {
    assert_eq!(left_of(&["a\\", "a"], &["a\\"]), ["a"]);
}

#[test]
fn a_pattern_that_is_longer_than_a_name_does_not_match_it() {
    assert_eq!(left_of(&["note"], &["notes"]), ["note"]);
}

#[test]
fn a_pattern_of_one_star_leaves_the_directory_empty() {
    assert_eq!(left_of(&FILES, &["*"]), Vec::<String>::new());
}

#[test]
fn a_pattern_that_matches_the_directory_itself_leaves_it_empty() {
    assert_eq!(left_of(&FILES, &["."]), Vec::<String>::new());
}

#[test]
fn a_file_that_is_kept_alone_is_left_as_it_is() {
    let (directory, tree) = tree_of(&[]);
    let file = tree.join("report.json");
    directory.write(&file, "{}").unwrap();
    artifacts::leave_out(&directory, &file, &patterns(&["*"]));
    assert!(directory.is_file(&file));
}

#[test]
fn a_symbolic_link_that_a_pattern_matches_is_left_out() {
    assert_eq!(linked_to_a_directory_apart(&["link"]).1, ["notes.txt"]);
}

#[test]
fn what_a_symbolic_link_that_is_left_out_leads_to_is_not_removed() {
    let (directory, _left) = linked_to_a_directory_apart(&["link"]);
    assert!(directory.is_file(&directory.path().join("apart/draft.tmp")));
}

#[test]
fn nothing_is_left_out_of_a_directory_that_a_symbolic_link_leads_to() {
    let (directory, _left) = linked_to_a_directory_apart(&["*.tmp"]);
    assert!(directory.is_file(&directory.path().join("apart/draft.tmp")));
}

#[test]
fn nothing_is_left_out_of_a_directory_that_is_kept_as_a_symbolic_link() {
    let (directory, tree) = tree_of(&["draft.tmp"]);
    let link = directory.path().join("link");
    directory.symlink(&tree, &link).unwrap();
    artifacts::leave_out(&directory, &link, &patterns(&["*"]));
    assert_eq!(everything_in(&directory, &tree), ["draft.tmp"]);
}
