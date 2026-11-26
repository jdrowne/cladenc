use std::time::Duration;

use crate::dataset::{CollectCommand, TaskConfiguration};
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{self, LAUNCH, TASK, Tried};

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const FIRST: [&str; 2] = ["exec", "first"];

fn collecting(commands: &[&str]) -> TaskConfiguration {
    TaskConfiguration {
        collect_commands: commands
            .iter()
            .map(|command| CollectCommand {
                command: (*command).to_owned(),
                user: None,
                time_limit: Duration::from_secs(30),
            })
            .collect(),
        ..TaskConfiguration::default()
    }
}

fn trialled(docker: Double, commands: &[&str]) -> Tried {
    given::trialled(docker, &LAUNCH, &given::environment(collecting(commands)))
}

fn place_of(tried: &Tried, command: &str) -> Option<usize> {
    let run = format!(" {command} 35");
    tried
        .commands()
        .iter()
        .position(|asked| asked.ends_with(&run))
}

#[test]
fn the_collect_commands_of_a_task_run_in_the_order_the_task_lists_them() {
    let tried = trialled(Double::default(), &["first", "second"]);
    assert_eq!(
        (place_of(&tried, "first"), place_of(&tried, "second")),
        (Some(9), Some(10))
    );
}

#[test]
fn a_collect_command_that_fails_does_not_keep_the_next_from_running() {
    let docker = Double::default().answering(&FIRST, &[Answer::Exited(3)]);
    let tried = trialled(docker, &["first", "second"]);
    assert_eq!(place_of(&tried, "second"), Some(10));
}

#[test]
fn a_collect_command_that_runs_past_its_time_limit_does_not_keep_the_next_from_running() {
    let docker = Double::default().answering(&FIRST, &[Answer::TimedOut]);
    let tried = trialled(docker, &["first", "second"]);
    assert_eq!(place_of(&tried, "second"), Some(10));
}

#[test]
fn the_work_of_the_harness_is_copied_out_before_a_collect_command_runs() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("\n/app\n")]);
    let tried = trialled(docker, &["first"]);
    let copied = tried
        .commands()
        .iter()
        .position(|asked| asked.starts_with("docker cp NAME:/app/. "));
    assert_eq!((copied, place_of(&tried, "first")), (Some(9), Some(10)));
}

#[test]
fn the_collect_commands_run_for_a_harness_that_was_stopped() {
    let docker = Double::default().answering(&HARNESS, &[Answer::TimedOut]);
    let tried = trialled(docker, &["first"]);
    assert!(place_of(&tried, "first").is_some());
}

#[test]
fn a_trial_whose_task_names_no_collect_command_keeps_nothing_of_one() {
    let tried = trialled(Double::default(), &[]);
    let collected = tried
        .outputs_directory
        .trial(0, TASK)
        .join("collection-hooks");
    assert!(!tried.directory.exists(collected));
}
