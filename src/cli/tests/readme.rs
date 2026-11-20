use std::collections::BTreeSet;
use std::fs;
use std::iter;
use std::path::Path;

use crate::cli::Cli;
use clap::{Arg, Command, CommandFactory};

const CLADENC: &str = "cladenc";
const HEAD_OF_SUB_COMMANDS: &str = "\n| Sub-command | Description |\n| --- | --- |\n";
const HEAD_OF_FLAGS: &str = "\n| Flag | Required | Description |\n| --- | --- | --- |\n";

fn readme() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md")).unwrap()
}

fn sub_commands() -> Vec<Command> {
    let mut sub_commands: Vec<Command> = Cli::command().get_subcommands().cloned().collect();
    sub_commands.sort_by(|one, other| one.get_name().cmp(other.get_name()));
    sub_commands
}

fn cladenc() -> Command {
    let mut cladenc = Cli::command();
    cladenc.build();
    cladenc
}

fn commands() -> Vec<Command> {
    iter::once(cladenc()).chain(sub_commands()).collect()
}

fn called(command: &Command) -> String {
    match command.get_name() {
        CLADENC => CLADENC.to_owned(),
        name => format!("{CLADENC} {name}"),
    }
}

fn heading_of(command: &Command) -> String {
    let level = if command.get_name() == CLADENC {
        "##"
    } else {
        "###"
    };
    format!("\n{level} `{}`\n", called(command))
}

fn link_to(command: &Command) -> String {
    match command.get_name() {
        CLADENC => format!("- [`{CLADENC}`](#{CLADENC}-1)"),
        name => format!("  - [`{CLADENC} {name}`](#{CLADENC}-{name})"),
    }
}

fn section_of(readme: &str, command: &Command) -> String {
    readme
        .split_once(&heading_of(command))
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

fn flags_in_order(command: &Command) -> Vec<String> {
    let mut flags: Vec<&Arg> = command.get_arguments().collect();
    flags.sort_by_key(|flag| name_of(flag));
    flags.into_iter().map(written).collect()
}

fn command_of(command: &Command) -> String {
    let lines: Vec<String> = iter::once(called(command))
        .chain(
            flags_in_order(command)
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

fn commands_whose_section(lacks: impl Fn(&Command, &str) -> bool) -> Vec<String> {
    let readme = readme();
    commands()
        .iter()
        .filter(|command| lacks(command, &section_of(&readme, command)))
        .map(called)
        .collect()
}

fn said_after_the_table_of(section: &str) -> Vec<String> {
    section
        .split_once(HEAD_OF_FLAGS)
        .map(|(_, rows)| {
            rows.lines()
                .skip_while(|line| line.starts_with('|'))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
        .trim()
        .split("\n\n")
        .map(str::to_owned)
        .collect()
}

fn is_a_list_of_steps(block: &str) -> bool {
    block
        .lines()
        .zip(1..)
        .all(|(line, number)| line.starts_with(&format!("{number}. ")))
}

fn sub_commands_whose_section(lacks: impl Fn(&Command, &[String]) -> bool) -> Vec<String> {
    let readme = readme();
    sub_commands()
        .iter()
        .filter(|sub_command| {
            lacks(
                sub_command,
                &said_after_the_table_of(&section_of(&readme, sub_command)),
            )
        })
        .map(called)
        .collect()
}

fn flags_of_cladenc() -> BTreeSet<String> {
    commands()
        .iter()
        .flat_map(|command| command.get_arguments().map(name_of).collect::<Vec<_>>())
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
fn the_table_of_contents_follows_the_first_paragraph_and_links_to_the_section_of_every_command() {
    let readme = readme();
    let contents = commands()
        .iter()
        .map(link_to)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(readme.split("\n\n").nth(2), Some(contents.as_str()));
}

#[test]
fn the_headings_under_the_title_are_cladenc_and_then_its_sub_commands_in_alphabetical_order() {
    assert_eq!(
        readme()
            .lines()
            .skip(1)
            .filter(|line| line.starts_with('#'))
            .map(|line| format!("\n{line}\n"))
            .collect::<Vec<_>>(),
        commands().iter().map(heading_of).collect::<Vec<_>>()
    );
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
fn the_table_of_sub_commands_lists_them_in_alphabetical_order() {
    assert_eq!(
        readme()
            .lines()
            .filter_map(|line| line.strip_prefix("| `cladenc "))
            .filter_map(|rest| rest.split_once('`'))
            .map(|(name, _)| name.to_owned())
            .collect::<Vec<_>>(),
        sub_commands()
            .iter()
            .map(|sub_command| sub_command.get_name().to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_section_of_a_command_starts_with_the_command_with_each_flag_on_a_line_of_its_own() {
    assert_eq!(
        commands_whose_section(|command, section| !section.starts_with(&command_of(command))),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_command_is_headed_flag_required_and_description() {
    assert_eq!(
        commands_whose_section(|_, section| !section.contains(HEAD_OF_FLAGS)),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_command_says_of_each_of_its_flags_whether_it_is_required() {
    assert_eq!(
        commands_whose_section(|command, section| {
            command
                .get_arguments()
                .any(|flag| !section.contains(&row_of(flag)))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_table_of_a_command_lists_its_flags_in_alphabetical_order() {
    assert_eq!(
        commands_whose_section(|command, section| {
            flags_with_a_row_in(section) != flags_in_order(command)
        }),
        Vec::<String>::new()
    );
}

#[test]
fn after_the_table_of_its_flags_the_section_of_a_sub_command_says_what_the_sub_command_does() {
    assert_eq!(
        sub_commands_whose_section(|sub_command, said| {
            !said
                .iter()
                .take(1)
                .any(|paragraph| paragraph.starts_with(&format!("`{}` ", called(sub_command))))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn what_a_sub_command_does_is_said_in_one_paragraph_of_one_line() {
    assert_eq!(
        sub_commands_whose_section(|_, said| {
            said.iter()
                .take(1)
                .any(|paragraph| paragraph.contains('\n'))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn nothing_follows_what_a_sub_command_does_but_one_numbered_list_of_its_steps() {
    assert_eq!(
        sub_commands_whose_section(|_, said| {
            said.len() > 2 || said.iter().skip(1).any(|block| !is_a_list_of_steps(block))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_section_of_every_sub_command_says_what_it_prints_on_standard_output() {
    assert_eq!(
        sub_commands_whose_section(|_, said| {
            !said
                .iter()
                .take(1)
                .any(|paragraph| paragraph.contains(" on standard output"))
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_section_of_cladenc_start_gives_its_steps_as_a_numbered_list() {
    assert_eq!(
        sub_commands_whose_section(|sub_command, said| {
            sub_command.get_name() == "start" && said.len() != 2
        }),
        Vec::<String>::new()
    );
}

#[test]
fn the_readme_names_no_budget() {
    assert!(!readme().contains("budget"));
}

#[test]
fn the_readme_names_no_directory_at_the_top_of_the_filesystem_of_a_container() {
    assert_eq!(
        ["`/harness`", "`/work`", "`/task`", "`/attempts`"]
            .into_iter()
            .filter(|top| readme().contains(top))
            .collect::<Vec<&str>>(),
        Vec::<&str>::new()
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

fn what_is_said_of(name: &str) -> String {
    let readme = readme();
    sub_commands()
        .iter()
        .find(|sub_command| sub_command.get_name() == name)
        .map(|sub_command| said_after_the_table_of(&section_of(&readme, sub_command)).join("\n"))
        .unwrap_or_default()
}

#[test]
fn the_table_of_sub_commands_says_that_report_is_for_a_directed_evolution_that_has_ended() {
    assert!(readme().contains(
        "\n| `cladenc report` | Prints the report of a directed evolution that has ended, "
    ));
}

#[test]
fn the_table_of_sub_commands_says_that_status_is_for_a_directed_evolution_that_has_not_ended() {
    assert!(readme().contains(
        "\n| `cladenc status` | Prints the state so far of a directed evolution that has not \
         ended, "
    ));
}

#[test]
fn the_section_of_cladenc_report_says_that_it_is_for_a_directed_evolution_that_has_ended() {
    assert!(
        what_is_said_of("report")
            .starts_with("`cladenc report` is for a directed evolution that has ended. ")
    );
}

#[test]
fn the_section_of_cladenc_report_says_what_it_does_with_a_directed_evolution_that_has_not_ended() {
    assert!(what_is_said_of("report").ends_with(
        "Given a directed evolution that has not ended, `cladenc report` prints nothing on \
         standard output: it says on standard error that the directed evolution has not ended \
         and that `cladenc status` is the sub-command to call, and exits with status 2."
    ));
}

#[test]
fn the_section_of_cladenc_status_says_that_it_is_for_a_directed_evolution_that_has_not_ended() {
    assert!(
        what_is_said_of("status")
            .starts_with("`cladenc status` is for a directed evolution that has not ended, ")
    );
}

#[test]
fn the_section_of_cladenc_status_says_what_it_does_with_a_directed_evolution_that_has_ended() {
    assert!(what_is_said_of("status").ends_with(
        "Given a directed evolution that has ended, `cladenc status` prints nothing on standard \
         output: it says on standard error that the directed evolution has ended and that \
         `cladenc report` is the sub-command to call, and exits with status 2."
    ));
}

#[test]
fn the_section_of_cladenc_status_says_that_what_it_prints_can_change() {
    assert!(what_is_said_of("status").contains(
        "What `cladenc status` prints is the state so far, which can change until the directed \
         evolution ends"
    ));
}

#[test]
fn the_section_of_cladenc_status_names_the_harness_that_is_ahead_so_far() {
    assert!(what_is_said_of("status").contains(
        "The harness that is ahead so far is the harness that would be named the best harness \
         if the directed evolution ended now"
    ));
}

#[test]
fn the_section_of_cladenc_start_says_that_it_prints_the_report_when_the_directed_evolution_ends() {
    assert!(
        what_is_said_of("start").contains(
            "when the directed evolution ends it prints there what `cladenc report` prints"
        )
    );
}

#[test]
fn the_section_of_cladenc_start_says_that_standard_output_holds_only_the_directory() {
    assert!(what_is_said_of("start").contains(
        "prints the directory of the best harness on standard output, and nothing else there."
    ));
}

#[test]
fn the_section_of_cladenc_start_says_that_a_directed_evolution_stopped_sooner_prints_no_report() {
    assert!(what_is_said_of("start").contains("prints no report and exits with status 1"));
}

#[test]
fn the_steps_of_cladenc_start_end_with_what_it_prints() {
    assert!(what_is_said_of("start").ends_with(
        "\n9. It prints on standard error what `cladenc report` prints, and on standard output \
         the directory of the best harness."
    ));
}

#[test]
fn the_section_of_cladenc_resume_says_that_it_prints_the_report_when_the_directed_evolution_ends() {
    assert!(what_is_said_of("resume").contains(
        "when the directed evolution ends it prints there what `cladenc report` prints, as \
         `cladenc start` does."
    ));
}

#[test]
fn the_section_of_cladenc_resume_says_what_it_prints_of_a_directed_evolution_that_had_ended() {
    assert!(what_is_said_of("resume").contains(
        "Of a directed evolution that had ended already, it prints the directory of the best \
         harness and the report again and does nothing else."
    ));
}
