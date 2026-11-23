use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::benchmark::{HealthCheck, TaskConfiguration};
use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{self, HEALTH_CHECK, LAUNCH, TIMER, Tried};
use crate::containers::{Checked, ContainerError, Launch, Retry};

const RETRIED: Launch<'static> = Launch {
    max_task_health_check_retries: 2,
    ..LAUNCH
};

const CHECK: [&str; 2] = ["exec", HEALTH_CHECK];
const REMOVED: &str = "docker rm --force NAME";

fn with(health_check: HealthCheck) -> TaskConfiguration {
    TaskConfiguration {
        health_check: Some(health_check),
        ..TaskConfiguration::default()
    }
}

fn checked(docker: Double, retries: u64) -> Tried {
    let environment = given::environment(with(given::checked(retries)));
    given::attempted(docker, &LAUNCH, &environment)
}

fn brought_up_again(docker: Double) -> Tried {
    let environment = given::environment(with(given::checked(1)));
    given::attempted(docker, &RETRIED, &environment)
}

fn never_healthy() -> Double {
    Double::default().answering(&CHECK, &[Answer::Exited(1)])
}

fn count_of(tried: &Tried, part: &str) -> usize {
    tried
        .commands()
        .iter()
        .filter(|command| command.contains(part))
        .count()
}

#[test]
fn the_health_check_of_a_task_runs_in_its_container_before_the_harness_is_put_there() {
    let tried = checked(Double::default(), 1);
    assert_eq!(
        tried.commands()[2],
        format!("docker exec --env-file /dev/stdin --user 0 NAME sh -c {TIMER} {HEALTH_CHECK} 35")
    );
}

#[test]
fn the_health_check_of_a_task_runs_as_the_user_of_the_image() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("builder\n/\n")]);
    let tried = checked(docker, 1);
    assert_eq!(
        given::following("--user", &tried.commands()[2]),
        ["builder"]
    );
}

#[test]
fn what_the_health_check_of_a_task_prints_is_kept_with_the_running_attempt() {
    let tried = checked(Double::default(), 1);
    assert_eq!(
        tried.docker.asked()[2].kept,
        tried.running().join("health-check")
    );
}

#[test]
fn the_container_of_a_task_with_a_health_check_stays_for_the_longest_the_check_can_take() {
    let tried = checked(Double::default(), 1);
    assert_eq!(tried.commands()[0].rsplit(' ').next(), Some("4294968555"));
}

#[test]
fn a_task_whose_health_check_passes_is_attempted() {
    assert!(checked(Double::default(), 1).result.is_ok());
}

#[test]
fn a_task_with_no_health_check_is_never_brought_up_again() {
    let environment = given::environment(TaskConfiguration::default());
    let tried = given::attempted(Double::default(), &RETRIED, &environment);
    assert!(tried.retries.is_empty());
}

#[test]
fn a_health_check_that_fails_is_run_again_as_often_as_the_task_allows() {
    let tried = checked(never_healthy(), 3);
    assert_eq!(count_of(&tried, HEALTH_CHECK), 3);
}

#[test]
fn a_health_check_that_passes_when_it_is_run_again_lets_the_attempt_go_on() {
    let docker = Double::default().answering(&CHECK, &[Answer::Exited(1), Answer::Exited(0)]);
    assert!(checked(docker, 3).result.is_ok());
}

#[test]
fn a_task_whose_health_check_does_not_pass_is_not_attempted() {
    let tried = checked(never_healthy(), 1);
    let printed = tried.running().join("health-check");
    assert!(matches!(
        &tried.result,
        Err(ContainerError::TaskHealthCheck { tries: 1, kept }) if *kept == printed
    ));
}

#[test]
fn the_container_of_a_task_whose_health_check_does_not_pass_is_removed() {
    let tried = checked(never_healthy(), 1);
    assert_eq!(tried.commands()[3..], [REMOVED]);
}

#[test]
fn a_task_whose_health_check_does_not_pass_is_brought_up_again_as_often_as_is_allowed() {
    let tried = brought_up_again(never_healthy());
    assert_eq!(count_of(&tried, "--detach"), 3);
}

#[test]
fn each_container_of_a_task_whose_health_check_does_not_pass_is_removed() {
    let tried = brought_up_again(never_healthy());
    assert_eq!(count_of(&tried, REMOVED), 3);
}

#[test]
fn each_time_a_task_is_brought_up_again_for_its_health_check_is_told() {
    let tried = brought_up_again(never_healthy());
    assert_eq!(
        tried.retries,
        [1, 2].map(|number| Retry {
            checked: Checked::Task,
            number,
            most: 2,
            wait: Duration::ZERO,
        })
    );
}

#[test]
fn the_wait_before_a_task_is_brought_up_again_starts_at_the_interval_of_its_check_and_doubles() {
    let health_check = HealthCheck {
        interval: Duration::from_millis(1),
        ..given::checked(1)
    };
    let environment = given::environment(with(health_check));
    let tried = given::attempted(never_healthy(), &RETRIED, &environment);
    let waits: Vec<Duration> = tried.retries.iter().map(|retry| retry.wait).collect();
    assert_eq!(waits, [Duration::from_millis(1), Duration::from_millis(2)]);
}

#[test]
fn a_task_whose_health_check_never_passes_names_how_often_it_was_brought_up() {
    let tried = brought_up_again(never_healthy());
    assert!(matches!(
        tried.result,
        Err(ContainerError::TaskHealthCheck { tries: 3, .. })
    ));
}

#[test]
fn a_task_whose_health_check_passes_once_it_is_brought_up_again_is_attempted() {
    let docker = Double::default().answering(&CHECK, &[Answer::Exited(1), Answer::Exited(0)]);
    assert!(brought_up_again(docker).result.is_ok());
}

#[test]
fn an_attempt_that_is_stopped_before_its_task_is_brought_up_again_was_stopped() {
    let stop = Arc::new(AtomicBool::new(false));
    let asked = Arc::clone(&stop);
    let docker = never_healthy().doing(&CHECK, move |_| asked.store(true, Ordering::SeqCst));
    let environment = given::environment(with(given::checked(1)));
    let tried = given::attempted_until(docker, &RETRIED, &environment, &stop);
    assert!(matches!(tried.result, Err(ContainerError::Stopped)));
}

#[test]
fn an_attempt_that_is_stopped_while_its_health_check_runs_was_stopped() {
    let docker = Double::default().answering(&CHECK, &[Answer::Stopped]);
    assert!(matches!(
        checked(docker, 1).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn a_health_check_that_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let tried = checked(
        Double::default().answering(&CHECK, &[Answer::Exited(125)]),
        1,
    );
    let said = tried.running().join("health-check/stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}
