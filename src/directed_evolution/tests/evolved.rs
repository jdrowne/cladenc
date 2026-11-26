use std::sync::atomic::{AtomicBool, Ordering};

use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;
use crate::sandboxes::tests::double::Double;

const IMAGES_REMOVED: &str = "docker image rm --force \
                              cladenc-environments-0123456789abcdef:0-verifier \
                              cladenc-environments-0123456789abcdef:0 \
                              cladenc-environments-0123456789abcdef:1-verifier \
                              cladenc-environments-0123456789abcdef:1";

const SHOWN_THE_PRIMOGENITOR: &str = "/trials/0:/var/log/cladenc/trials:ro";
const EXPERIMENT: &str = "the singleton control group harness, by the primogenitor from a \
                          second clone of harness 1";

fn evolved() -> Evolved {
    given::started(Double::default(), given::unchanged)
}

fn changing() -> Evolved {
    given::started(Double::default().changing(), given::unchanged)
}

fn with_three_diversifications(_: &Memory, settings: &mut Settings) {
    settings.max_diversifications = 3;
}

fn experimented() -> Evolved {
    let diversified = AtomicBool::new(false);
    let docker = Double::default().changing_when(move |asked| {
        let by_the_primogenitor = asked
            .arguments
            .iter()
            .any(|argument| argument.ends_with(SHOWN_THE_PRIMOGENITOR));
        !(by_the_primogenitor && diversified.swap(true, Ordering::SeqCst))
    });
    given::started(docker, with_three_diversifications)
}

fn with_a_selection_task(_: &Memory, settings: &mut Settings) {
    settings.holdout_tasks = vec!["dataset/t1".to_owned()];
}

fn selected() -> Evolved {
    let docker = Double::default()
        .changing()
        .verifying(|trial| trial.starts_with("trial-1-"));
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
fn the_primogenitor_trials_each_task_in_a_sandbox_of_its_own() {
    assert_eq!(evolved().count_of("--detach"), 2);
}

#[test]
fn a_diversification_starts_once_the_primogenitor_trialled_every_task() {
    let said = evolved().said;
    assert!(said.contains(
        "started the diversification of harness 1 by harness 0, diversification \
                           1 of 1"
    ));
}

#[test]
fn a_progenitor_that_leaves_its_clone_unchanged_creates_no_harness() {
    let said = evolved().said;
    assert!(said.contains(
        "the diversification of harness 1 by harness 0 left the clone of harness 0 \
         unchanged, and created no harness"
    ));
}

#[test]
fn the_images_of_the_environments_are_removed_once_the_directed_evolution_ends() {
    let commands = evolved().commands();
    assert_eq!(commands.last().map(String::as_str), Some(IMAGES_REMOVED));
}

#[test]
fn a_clone_that_its_progenitor_diversifies_becomes_a_harness() {
    let said = changing().said;
    assert!(said.contains(
        "the diversification of harness 1 by harness 0 created harness 1, kept \
                           in"
    ));
}

#[test]
fn a_child_trials_every_task_as_well() {
    assert_eq!(changing().count_of("--detach"), 4);
}

#[test]
fn a_directed_evolution_ends_with_the_count_of_its_harnesses() {
    let said = changing().said;
    assert!(said.contains("and 2 harnesses: its best harness is harness 0"));
}

#[test]
fn an_experiment_starts_when_a_harness_other_than_the_primogenitor_is_cloned() {
    assert!(experimented().said.contains(EXPERIMENT));
}

#[test]
fn the_singleton_control_group_harness_of_an_experiment_is_kept_among_the_harnesses() {
    let said = experimented().said;
    assert!(said.contains(&format!("{EXPERIMENT} created harness")));
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
