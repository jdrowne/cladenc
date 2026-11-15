use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use cladenc::cli::Cli;
use clap::{Arg, Command, CommandFactory};

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

fn written(option: &Arg) -> String {
    let long = option.get_long().unwrap_or_default();
    match option.get_value_names() {
        Some(names) if option.get_action().takes_values() => {
            format!("`--{long} <{}>`", names[0])
        }
        _ => format!("`--{long}`"),
    }
}

fn row_of(option: &Arg) -> String {
    let required = if option.is_required_set() {
        "yes"
    } else {
        "no"
    };
    format!("\n| {} | {required} | ", written(option))
}

fn rows_looked_for() -> Vec<(String, bool)> {
    let readme = readme();
    sub_commands()
        .iter()
        .flat_map(|sub_command| {
            let section = section_of(&readme, sub_command);
            sub_command
                .get_arguments()
                .map(|option| {
                    let row = row_of(option);
                    let found = section.contains(&row);
                    (format!("{}:{row}", sub_command.get_name()), found)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn options_of_cladenc() -> BTreeSet<String> {
    sub_commands()
        .iter()
        .flat_map(|sub_command| {
            sub_command
                .get_arguments()
                .map(|option| format!("--{}", option.get_long().unwrap_or_default()))
                .collect::<Vec<_>>()
        })
        .chain(["--help".to_owned(), "--version".to_owned()])
        .collect()
}

fn options_named_in(readme: &str) -> BTreeSet<String> {
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
fn the_section_of_a_sub_command_has_a_row_for_each_of_its_options_that_says_whether_it_is_required()
{
    assert_eq!(
        rows_looked_for()
            .iter()
            .filter(|(_, found)| !found)
            .collect::<Vec<_>>(),
        Vec::<&(String, bool)>::new()
    );
}

#[test]
fn the_readme_names_no_option_that_cladenc_does_not_have() {
    assert_eq!(
        options_named_in(&readme())
            .difference(&options_of_cladenc())
            .collect::<Vec<_>>(),
        Vec::<&String>::new()
    );
}
