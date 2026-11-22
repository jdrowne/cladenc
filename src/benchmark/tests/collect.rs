use std::fs;
use std::time::Duration;

use crate::benchmark::configuration::{self, ConfigurationError};
use crate::benchmark::{CollectCommand, TaskConfiguration};

const SNAPSHOT: &str = "[[verifier.collect]]\ncommand = \"pg_dump app > /tmp/rows.sql\"\n";
const ELSEWHERE: &str = "a command in a container other than that of the harness";

fn read(written: &str) -> Result<TaskConfiguration, ConfigurationError> {
    let directory = test_inputs::temporary_directory();
    fs::write(directory.path().join("task.toml"), written).unwrap();
    configuration::read_with(directory.path(), &|_| None)
}

fn commands(written: &str) -> Vec<CollectCommand> {
    read(written).unwrap().collect_commands
}

fn snapshot() -> CollectCommand {
    CollectCommand {
        command: "pg_dump app > /tmp/rows.sql".to_owned(),
        user: None,
        time_limit: Duration::from_secs(60),
    }
}

fn why_it_is_not_valid(written: &str) -> Option<String> {
    match read(written) {
        Err(ConfigurationError::NotValid { reason, .. }) => Some(reason),
        _ => None,
    }
}

#[test]
fn a_task_that_lists_no_command_under_collect_has_none_run() {
    assert_eq!(commands(""), Vec::new());
}

#[test]
fn a_command_under_collect_is_read_with_60_seconds_and_no_user_where_it_gives_neither() {
    assert_eq!(commands(SNAPSHOT), [snapshot()]);
}

#[test]
fn the_time_limit_of_a_command_under_collect_is_read_from_its_timeout_sec() {
    assert_eq!(
        commands(&format!("{SNAPSHOT}timeout_sec = 2.5\n"))[0].time_limit,
        Duration::from_millis(2500)
    );
}

#[test]
fn the_user_of_a_command_under_collect_is_read_as_a_name() {
    assert_eq!(
        commands(&format!("{SNAPSHOT}user = \"postgres\"\n"))[0]
            .user
            .as_deref(),
        Some("postgres")
    );
}

#[test]
fn the_user_of_a_command_under_collect_is_read_as_a_number() {
    assert_eq!(
        commands(&format!("{SNAPSHOT}user = 1000\n"))[0]
            .user
            .as_deref(),
        Some("1000")
    );
}

#[test]
fn a_user_of_a_command_under_collect_named_with_nothing_is_left_out() {
    assert_eq!(commands(&format!("{SNAPSHOT}user = \"\"\n"))[0].user, None);
}

#[test]
fn the_commands_under_collect_are_read_in_the_order_the_task_lists_them() {
    assert_eq!(
        commands(&format!(
            "{SNAPSHOT}\n[[verifier.collect]]\ncommand = \"sync\"\n"
        ))
        .iter()
        .map(|command| command.command.as_str())
        .collect::<Vec<_>>(),
        ["pg_dump app > /tmp/rows.sql", "sync"]
    );
}

#[test]
fn a_command_under_collect_written_with_nothing_is_left_out() {
    assert_eq!(
        commands("[[verifier.collect]]\ncommand = \"\"\n"),
        Vec::new()
    );
}

#[test]
fn a_command_under_collect_for_the_container_of_the_harness_is_not_refused() {
    assert_eq!(
        commands(&format!("{SNAPSHOT}service = \"main\"\n")),
        [snapshot()]
    );
}

#[test]
fn a_command_under_collect_whose_service_is_written_with_nothing_is_not_refused() {
    assert_eq!(
        commands(&format!("{SNAPSHOT}service = \"\"\n")),
        [snapshot()]
    );
}

#[test]
fn a_command_under_collect_for_another_container_is_refused() {
    assert_eq!(
        read(&format!("{SNAPSHOT}service = \"api\"\n")),
        Err(ConfigurationError::Needs { what: ELSEWHERE })
    );
}

#[test]
fn a_time_limit_of_0_for_a_command_under_collect_is_refused() {
    assert_eq!(
        read(&format!("{SNAPSHOT}timeout_sec = 0\n")),
        Err(ConfigurationError::NotATimeLimit {
            key: "verifier.collect.timeout_sec"
        })
    );
}

#[test]
fn an_entry_under_collect_that_names_no_command_is_not_valid() {
    assert_eq!(
        why_it_is_not_valid("[[verifier.collect]]\ntimeout_sec = 30\n"),
        Some("missing field `command`".to_owned())
    );
}

#[test]
fn a_command_under_collect_that_is_not_text_is_not_valid() {
    assert_eq!(
        why_it_is_not_valid("[[verifier.collect]]\ncommand = 5\n"),
        Some("invalid type: integer `5`, expected a string".to_owned())
    );
}

#[test]
fn a_time_limit_of_a_command_under_collect_that_is_not_a_number_is_not_valid() {
    assert!(
        why_it_is_not_valid(&format!("{SNAPSHOT}timeout_sec = \"soon\"\n"))
            .is_some_and(|reason| reason.starts_with("invalid type: string \"soon\""))
    );
}

#[test]
fn a_command_in_another_container_is_said_with_what_the_task_asks_for() {
    assert_eq!(
        ConfigurationError::Needs { what: ELSEWHERE }.to_string(),
        "asks for a command in a container other than that of the harness, which cladenc has no \
         way to give"
    );
}
