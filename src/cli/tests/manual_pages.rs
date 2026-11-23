use std::iter;

use crate::cli::Cli;
use clap::{Arg, ArgAction, Command, CommandFactory};
use test_inputs::{manual, manual_on_one_line};

fn sub_commands() -> Vec<(String, Command)> {
    Cli::command()
        .get_subcommands()
        .map(|sub_command| {
            let page = format!("cladenc-{}", sub_command.get_name());
            (page, sub_command.clone())
        })
        .collect()
}

fn commands() -> Vec<(String, Command)> {
    iter::once(("cladenc".to_owned(), Cli::command()))
        .chain(sub_commands())
        .collect()
}

fn pages_without(said: impl Fn(&str, &Command) -> String) -> Vec<String> {
    commands()
        .iter()
        .filter(|(page, command)| !manual_on_one_line(page).contains(&said(page, command)))
        .map(|(page, _)| page.clone())
        .collect()
}

fn name_of(page: &str, command: &Command) -> String {
    let about = command.get_about().unwrap().to_string();
    let (first, rest) = about.split_at(1);
    format!("NAME\n{page} - {}{rest}\n", first.to_lowercase())
}

fn tag_of(flag: &Arg) -> String {
    let name = format!("--{}", flag.get_long().unwrap());
    if flag.get_action().takes_values() {
        let value = flag.get_value_names().unwrap()[0].as_str();
        format!("{name} <{value}>")
    } else {
        name
    }
}

fn said_of(flag: &Arg) -> String {
    format!("{} {}.", tag_of(flag), flag.get_help().unwrap())
}

fn in_the_synopsis(flag: &Arg) -> String {
    let tag = tag_of(flag);
    let tag = if matches!(flag.get_action(), ArgAction::Append) {
        format!("{tag}...")
    } else {
        tag
    };
    if flag.is_required_set() {
        tag
    } else {
        format!("[{tag}]")
    }
}

fn synopsis_from(sub_command: &Command) -> Vec<String> {
    let mut flags: Vec<(String, String)> = sub_command
        .get_arguments()
        .map(|flag| (flag.get_long().unwrap().to_owned(), in_the_synopsis(flag)))
        .collect();
    flags.push(("help".to_owned(), "[--help]".to_owned()));
    flags.sort();
    iter::once(format!("cladenc {}", sub_command.get_name()))
        .chain(flags.into_iter().map(|(_, flag)| flag))
        .collect()
}

fn synopsis_of(page: &str) -> Vec<String> {
    manual(page)
        .lines()
        .skip_while(|line| *line != "SYNOPSIS")
        .skip(1)
        .take_while(|line| *line != "DESCRIPTION")
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn package() -> String {
    test_inputs::PACKAGE.to_owned()
}

#[test]
fn every_manual_page_is_named_with_what_the_help_says_its_command_does() {
    let unnamed: Vec<String> = commands()
        .iter()
        .filter(|(page, command)| !manual(page).contains(&name_of(page, command)))
        .map(|(page, _)| page.clone())
        .collect();
    assert_eq!(unnamed, Vec::<String>::new());
}

#[test]
fn the_manual_page_of_every_sub_command_gives_each_flag_with_what_the_help_says_of_it() {
    let missing: Vec<String> = sub_commands()
        .iter()
        .flat_map(|(page, sub_command)| {
            let said = manual_on_one_line(page);
            sub_command
                .get_arguments()
                .map(said_of)
                .filter(|flag| !said.contains(flag.as_str()))
                .collect::<Vec<String>>()
        })
        .collect();
    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn the_synopsis_of_every_sub_command_gives_each_flag_and_brackets_those_not_required() {
    let (written, expected): (Vec<Vec<String>>, Vec<Vec<String>>) = sub_commands()
        .iter()
        .map(|(page, sub_command)| (synopsis_of(page), synopsis_from(sub_command)))
        .unzip();
    assert_eq!(written, expected);
}

#[test]
fn every_manual_page_gives_the_flag_that_prints_help() {
    let without = pages_without(|_, _| " -h, --help Print help. ".to_owned());
    assert_eq!(without, Vec::<String>::new());
}

#[test]
fn the_manual_page_of_cladenc_gives_the_flag_that_prints_the_version() {
    assert!(manual_on_one_line("cladenc").contains(" -V, --version Print version. "));
}

#[test]
fn the_manual_page_of_cladenc_names_the_manual_page_of_every_sub_command() {
    let said = manual_on_one_line("cladenc");
    let unnamed: Vec<String> = sub_commands()
        .iter()
        .map(|(page, _)| format!("Its manual page is {page}(1)."))
        .filter(|named| !said.contains(named.as_str()))
        .collect();
    assert_eq!(unnamed, Vec::<String>::new());
}

#[test]
fn every_manual_page_ends_with_the_other_manual_pages_to_see_also() {
    let mut pages: Vec<String> = commands().iter().map(|(page, _)| page.clone()).collect();
    pages.sort();
    let without: Vec<&String> = pages
        .iter()
        .filter(|page| {
            let others: Vec<String> = pages
                .iter()
                .filter(|other| other != page)
                .map(|other| format!("{other}(1)"))
                .collect();
            !manual_on_one_line(page).ends_with(&format!("SEE ALSO {}", others.join(", ")))
        })
        .collect();
    assert_eq!(without, Vec::<&String>::new());
}

#[test]
fn the_package_installs_every_manual_page_where_the_standard_puts_those_of_an_add_on_package() {
    let package = package();
    let not_installed: Vec<String> = commands()
        .iter()
        .map(|(page, _)| page.clone())
        .filter(|page| {
            !package.contains(&format!(
                "  - dst: /opt/cladenc/share/man/man1/{page}.1\n    file_info:\n      mode: \
                 0644\n    src: docs/man/man1/{page}.1\n"
            ))
        })
        .collect();
    assert_eq!(not_installed, Vec::<String>::new());
}
