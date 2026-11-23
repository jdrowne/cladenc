use std::path::Path;

use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Settings;

const CHECKED: &str = "[environment.healthcheck]\ncommand = \"test -e /ready\"\ninterval_sec = \
                       0.0\nretries = 1\n";

fn with_a_health_check(directory: &Path, settings: &mut Settings) {
    settings.max_task_health_check_retries = 1;
    given::configured(directory, "t1", CHECKED);
}

fn brought_up_again() -> Evolved {
    let answers = [Answer::Exited(1), Answer::Exited(0)];
    let docker = Double::default().answering(&["exec", "test -e /ready"], &answers);
    given::started(docker, with_a_health_check)
}

#[test]
fn a_task_that_is_brought_up_again_for_its_health_check_is_told() {
    let said = brought_up_again().said;
    assert!(said.contains(
        "the attempt of harness 0 on task benchmark/t1 is not carried out yet: the health check \
         of the task did not pass"
    ));
}

#[test]
fn a_task_that_is_brought_up_again_is_told_with_the_number_of_the_retry() {
    let said = brought_up_again().said;
    assert!(said.contains("retry 1 of 1"));
}

#[test]
fn a_task_that_is_brought_up_again_gets_a_new_container() {
    assert_eq!(brought_up_again().count_of("--detach"), 3);
}

#[test]
fn a_directed_evolution_goes_on_once_the_health_check_of_a_task_passes() {
    let best = brought_up_again()
        .result
        .ok()
        .map(|found| found.best_harness);
    assert_eq!(best, Some(0));
}
