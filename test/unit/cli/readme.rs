use std::collections::BTreeSet;
use std::fs;
use std::iter;
use std::path::Path;

use cladenc::cli::Cli;
use clap::{Arg, Command, CommandFactory};

const HEAD_OF_SUB_COMMANDS: &str = "\n| Sub-command | Description |\n| --- | --- |\n";
const HEAD_OF_FLAGS: &str = "\n| Flag | Required | Description |\n| --- | --- | --- |\n";

fn readme() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md")).unwrap()
}

fn sub_commands() -> Vec<Command> {
    Cli::command().get_subcommands().cloned().collect()
}

fn heading_of(sub_command: &Command) -> String {
    format!("\n### `cladenc {}`\n", sub_command.get_name())
}

fn section_of(readme: &str, sub_command: &Command) -> String {
    readme
        .split_once(&heading_of(sub_command))
        .map(|(_, after)| after.split("\n##").next().unwrap_or_default().to_owned())
        .unwrap_or_default()
}

fn name_of(flag: &Arg) -> String {
    format!("--{}", flag.get_long().unwrap_or_default())
}

fn written(flag: &Arg) -> String {
    match flag.get_value_names() {
        Some(names) if flag.get_action().takes_values() => {
            format!("{} <{}>", name_of(flag), names[0])
        }
        _ => name_of(flag),
    }
}

fn flags_in_order(sub_command: &Command) -> Vec<String> {
    let mut flags: Vec<&Arg> = sub_command.get_arguments().collect();
    flags.sort_by_key(|flag| name_of(flag));
    flags.into_iter().map(written).collect()
}

fn command_of(sub_command: &Command) -> String {
    let lines: Vec<String> = iter::once(format!("cladenc {}", sub_command.get_name()))
        .chain(
            flags_in_order(sub_command)
                .iter()
                .map(|flag| format!("  {flag}")),
        )
        .collect();
    format!("\n```text\n{}\n```\n", lines.join(" \\\n"))
}

fn row_of(flag: &Arg) -> String {
    let required = if flag.is_required_set() {
        "Required"
    } else {
        "Not Required"
    };
    format!("\n| `{}` | {required} | ", written(flag))
}

fn flags_with_a_row_in(section: &str) -> Vec<String> {
    section
        .lines()
        .filter_map(|line| line.strip_prefix("| `--"))
        .filter_map(|rest| rest.split_once('`'))
        .map(|(flag, _)| format!("--{flag}"))
        .collect()
}

fn sub_commands_whose_section(lacks: impl Fn(&Command, &str) -> bool) -> Vec<String> {
    let readme = readme();
    sub_commands()
        .iter()
        .filter(|sub_command| lacks(sub_command, &section_of(&readme, sub_command)))
        .map(|sub_command| sub_command.get_name().to_owned())
        .collect()
}

fn flags_of_cladenc() -> BTreeSet<String> {
    sub_commands()
        .iter()
        .flat_map(|sub_command| sub_command.get_arguments().map(name_of).collect::<Vec<_>>())
        .chain(["--help".to_owned(), "--version".to_owned()])
        .collect()
}

fn flags_named_in(readme: &str) -> BTreeSet<String> {
    readme
        .split(|character: char| !(character.is_ascii_lowercase() || character == '-'))
        .filter(|word| {
            word.strip_prefix("--")
                .is_some_and(|name| name.starts_with(|first: char| first.is_ascii_lowercase()))
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_table_of_sub_commands_is_headed_sub_command_and_description() {
    assert!(readme().contains(HEAD_OF_SUB_COMMANDS));
}

#[test]
fn the_table_of_sub_commands_has_a_row_for_every_sub_command() {
    let readme = readme();
    assert_eq!(
        sub_commands()
            .iter()
            .map(|sub_command| format!("\n| `cladenc {}` | ", sub_command.get_name()))
            .filter(|row| !readme.contains(row))
            .collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn the_readme_has_a_section_for_every_sub_command() {
    let readme = readme();
    assert_eq!(
        sub_commands()
            .iter()
            .map(heading_of)
            .filter(|heading| !readme.contains(heading))
            .collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn the_section_of_a_sub_command_starts_with_its_command_with_each_flag_on_a_line_of_its_own() {
    assert_eq!(
        sub_commands_whose_section(|sub_command, section| {
            !section.starts_with(&command_of(sub_command))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_sub_command_is_headed_flag_required_and_description() {
    assert_eq!(
        sub_commands_whose_section(|_, section| !section.contains(HEAD_OF_FLAGS)),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_sub_command_says_of_each_of_its_flags_whether_it_is_required() {
    assert_eq!(
        sub_commands_whose_section(|sub_command, section| {
            sub_command
                .get_arguments()
                .any(|flag| !section.contains(&row_of(flag)))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_sub_command_lists_its_flags_in_alphabetical_order() {
    assert_eq!(
        sub_commands_whose_section(|sub_command, section| {
            flags_with_a_row_in(section) != flags_in_order(sub_command)
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_readme_names_no_flag_that_cladenc_does_not_have() {
    assert_eq!(
        flags_named_in(&readme())
            .difference(&flags_of_cladenc())
            .collect::<Vec<_>>(),
        Vec::<&String>::new()
    );
}
