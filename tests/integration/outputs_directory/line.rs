use std::fs;
use std::path::Path;

use cladenc::genealogy::CreationOutcome;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    (directory, outputs_directory)
}

fn procreated(
    outputs_directory: &OutputsDirectory,
    child: u64,
    progenitor: u64,
    outcome: CreationOutcome,
) {
    outputs_directory
        .start_procreation(child, progenitor)
        .unwrap();
    outputs_directory
        .end_procreation(child, progenitor, outcome)
        .unwrap();
}

fn family() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    procreated(&outputs_directory, 1, 0, CreationOutcome::Created);
    procreated(&outputs_directory, 2, 1, CreationOutcome::Created);
    procreated(&outputs_directory, 3, 0, CreationOutcome::Failed);
    procreated(&outputs_directory, 4, 2, CreationOutcome::Created);
    outputs_directory.start_procreation(5, 1).unwrap();
    (directory, outputs_directory)
}

fn with_eleven_children_of_the_primogenitor() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    for child in 1..=11 {
        procreated(&outputs_directory, child, 0, CreationOutcome::Created);
    }
    (directory, outputs_directory)
}

fn with_two_records_that_name_each_other() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = family();
    fs::write(outputs_directory.procreation(1).join("progenitor"), "2\n").unwrap();
    (directory, outputs_directory)
}

#[test]
fn the_primogenitor_has_no_progenitor() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.progenitor_of(0), None);
}

#[test]
fn the_progenitor_of_a_child_is_the_harness_that_made_it() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.progenitor_of(2), Some(1));
}

#[test]
fn the_progenitor_of_a_child_that_was_not_created_is_the_harness_that_made_it() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.progenitor_of(3), Some(0));
}

#[test]
fn a_child_whose_procreation_has_not_ended_has_no_progenitor_on_record() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.progenitor_of(5), None);
}

#[test]
fn a_record_that_does_not_hold_an_id_names_no_progenitor() {
    let (_directory, outputs_directory) = family();
    fs::write(
        outputs_directory.procreation(1).join("progenitor"),
        "none\n",
    )
    .unwrap();
    assert_eq!(outputs_directory.progenitor_of(1), None);
}

#[test]
fn the_primogenitor_descends_from_no_harness() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.line_of(0), Vec::<u64>::new());
}

#[test]
fn a_harness_made_by_the_primogenitor_descends_from_it_alone() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.line_of(1), [0]);
}

#[test]
fn the_line_of_a_harness_runs_from_its_progenitor_back_to_the_primogenitor() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.line_of(4), [2, 1, 0]);
}

#[test]
fn a_line_ends_at_a_record_that_names_a_later_harness_as_the_progenitor_of_an_earlier_one() {
    let (_directory, outputs_directory) = with_two_records_that_name_each_other();
    assert_eq!(outputs_directory.line_of(4), [2, 1]);
}

#[test]
fn the_children_of_a_harness_are_those_it_made_whether_they_were_created_or_not() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.children_of(0), [1, 3]);
}

#[test]
fn a_child_whose_procreation_has_not_ended_is_not_among_the_children_of_its_progenitor() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.children_of(1), [2]);
}

#[test]
fn a_harness_that_has_made_no_child_has_none() {
    let (_directory, outputs_directory) = family();
    assert_eq!(outputs_directory.children_of(4), Vec::<u64>::new());
}

#[test]
fn the_children_of_a_harness_are_in_the_order_of_their_ids_and_not_of_their_names() {
    let (_directory, outputs_directory) = with_eleven_children_of_the_primogenitor();
    assert_eq!(
        outputs_directory.children_of(0),
        (1..=11).collect::<Vec<u64>>()
    );
}

#[test]
fn what_is_kept_beside_the_procreations_under_a_name_that_is_no_id_is_no_child() {
    let (_directory, outputs_directory) = family();
    fs::create_dir(outputs_directory.path().join("procreations/notes")).unwrap();
    assert_eq!(outputs_directory.children_of(0), [1, 3]);
}

#[test]
fn an_outputs_directory_that_is_not_there_has_no_children_to_list() {
    assert_eq!(
        OutputsDirectory::at(Path::new("/nowhere/outputs")).children_of(0),
        Vec::<u64>::new()
    );
}
