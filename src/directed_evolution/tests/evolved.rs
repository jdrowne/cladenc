use std::path::Path;

use crate::containers::tests::double::Double;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Settings;

const IMAGES_REMOVED: &str = "docker image rm --force \
                              cladenc-environments-0123456789abcdef:0-grader \
                              cladenc-environments-0123456789abcdef:0 \
                              cladenc-environments-0123456789abcdef:1-grader \
                              cladenc-environments-0123456789abcdef:1";

fn evolved() -> Evolved {
    given::started(Double::default(), given::unchanged)
}

fn changing() -> Evolved {
    given::started(Double::default().changing(), given::unchanged)
}

fn with_a_selection_task(_: &Path, settings: &mut Settings) {
    settings.holdout_tasks = vec!["benchmark/t1".to_owned()];
}

fn selected() -> Evolved {
    let docker = Double::default()
        .changing()
        .grading(|attempt| attempt.starts_with("attempt-1-"));
    given::started(docker, with_a_selection_task)
}

#[test]
fn a_directed_evolution_whose_harnesses_complete_no_task_finds_the_primogenitor_best() {
    let best = evolved().result.ok().map(|found| found.best_harness);
    assert_eq!(best, Some(0));
}

#[test]
fn the_best_harness_is_found_among_the_harnesses_of_the_outputs_directory() {
    let found = evolved().result;
    assert!(found.is_ok_and(|found| found.directory.ends_with("outputs/harnesses/0")));
}

#[test]
fn the_primogenitor_attempts_each_task_in_a_container_of_its_own() {
    assert_eq!(evolved().count_of("--detach"), 2);
}

#[test]
fn a_procreation_starts_once_the_primogenitor_attempted_every_task() {
    let said = evolved().said;
    assert!(said.contains("started the procreation of harness 1 by harness 0, procreation 1 of 1"));
}

#[test]
fn a_progenitor_that_leaves_its_copy_unchanged_makes_no_harness() {
    let said = evolved().said;
    assert!(said.contains("and 1 harness: its best harness is harness 0"));
}

#[test]
fn the_images_of_the_environments_are_removed_once_the_directed_evolution_ends() {
    let commands = evolved().commands();
    assert_eq!(commands.last().map(String::as_str), Some(IMAGES_REMOVED));
}

#[test]
fn a_copy_that_its_progenitor_changes_becomes_a_harness_with_a_singleton_control_group_harness() {
    let said = changing().said;
    assert!(
        said.contains("started the creation of harness 2, the singleton control group harness")
    );
}

#[test]
fn a_child_and_its_singleton_control_group_harness_attempt_every_task_as_well() {
    assert_eq!(changing().count_of("--detach"), 6);
}

#[test]
fn a_directed_evolution_ends_with_each_harness_that_was_made() {
    let said = changing().said;
    assert!(said.contains("and 3 harnesses: its best harness is harness 0"));
}

#[test]
fn the_harness_that_completes_the_most_selection_tasks_is_the_best() {
    let best = selected().result.ok().map(|found| found.best_harness);
    assert_eq!(best, Some(1));
}

#[test]
fn the_best_harness_is_kept_in_a_directory_of_its_own() {
    let found = selected().result;
    assert!(found.is_ok_and(|found| found.directory.ends_with("outputs/harnesses/1")));
}
