use std::path::PathBuf;

use crate::containers::shown;
use crate::file_system::tests::double::Memory;
use crate::genealogy::{CreationOutcome, HarnessId};
use crate::outputs_directory::OutputsDirectory;

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

fn diversify(
    outputs_directory: &OutputsDirectory,
    child: HarnessId,
    progenitor: HarnessId,
    outcome: CreationOutcome,
) {
    outputs_directory
        .start_diversification(child, progenitor)
        .unwrap();
    outputs_directory
        .end_diversification(child, progenitor, outcome)
        .unwrap();
}

fn with_three_generations() -> (Memory, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    diversify(&outputs_directory, 1, 0, CreationOutcome::Created);
    diversify(&outputs_directory, 2, 1, CreationOutcome::Created);
    diversify(&outputs_directory, 3, 1, CreationOutcome::Failed);
    diversify(&outputs_directory, 4, 1, CreationOutcome::Created);
    outputs_directory.start_build(2).unwrap();
    (directory, outputs_directory)
}

fn names(given: Vec<(PathBuf, String)>) -> Vec<String> {
    given.into_iter().map(|(_kept, name)| name).collect()
}

#[test]
fn a_progenitor_is_shown_every_harness_it_descends_from_with_the_nearest_first() {
    let (_directory, outputs_directory) = with_three_generations();
    assert_eq!(
        names(shown::to(&outputs_directory, 2, None)),
        [
            "/var/log/cladenc/line/1/harness",
            "/var/log/cladenc/line/1/trials",
            "/var/log/cladenc/line/1/diversification",
            "/var/log/cladenc/line/0/harness",
            "/var/log/cladenc/line/0/trials",
        ]
    );
}

#[test]
fn a_progenitor_is_shown_every_child_it_made_after_its_line_whether_created_or_not() {
    let (_directory, outputs_directory) = with_three_generations();
    assert_eq!(
        names(shown::to(&outputs_directory, 1, None)),
        [
            "/var/log/cladenc/line/0/harness",
            "/var/log/cladenc/line/0/trials",
            "/var/log/cladenc/children/2/harness",
            "/var/log/cladenc/children/2/trials",
            "/var/log/cladenc/children/2/diversification",
            "/var/log/cladenc/children/2/build",
            "/var/log/cladenc/children/3/harness",
            "/var/log/cladenc/children/3/diversification",
            "/var/log/cladenc/children/4/harness",
            "/var/log/cladenc/children/4/trials",
            "/var/log/cladenc/children/4/diversification",
        ]
    );
}

#[test]
fn a_progenitor_shown_what_it_saw_before_a_child_is_shown_only_the_children_made_before_it() {
    let (_directory, outputs_directory) = with_three_generations();
    assert_eq!(
        names(shown::to(&outputs_directory, 1, Some(3))),
        [
            "/var/log/cladenc/line/0/harness",
            "/var/log/cladenc/line/0/trials",
            "/var/log/cladenc/children/2/harness",
            "/var/log/cladenc/children/2/trials",
            "/var/log/cladenc/children/2/diversification",
            "/var/log/cladenc/children/2/build",
        ]
    );
}

#[test]
fn what_a_progenitor_is_shown_of_a_harness_is_what_the_outputs_directory_keeps_of_it() {
    let (_directory, outputs_directory) = with_three_generations();
    let kept: Vec<PathBuf> = shown::to(&outputs_directory, 0, None)
        .into_iter()
        .map(|(kept, _name)| kept)
        .collect();
    assert_eq!(
        kept,
        [
            outputs_directory.harness(1),
            outputs_directory.trials(1),
            outputs_directory.diversification(1),
        ]
    );
}

#[test]
fn the_primogenitor_is_shown_no_harness_before_it_has_made_a_child() {
    let (_directory, outputs_directory) = made();
    assert!(shown::to(&outputs_directory, 0, None).is_empty());
}
