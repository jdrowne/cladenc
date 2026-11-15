use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found};
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        tasks: vec!["t1".to_owned()],
        expander: test_inputs::sample("integration", "expand-solve-one-more.sh"),
        evaluator: test_inputs::sample("integration", "evaluate-listed.sh"),
        budget: 10,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_expansions: 0,
        workers: 1,
        expansion_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn ended() -> (TempDir, PathBuf, Found) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(false), None).unwrap();
    (directory, run_directory, found)
}

#[test]
fn resuming_a_search_that_ended_returns_what_the_search_returned() {
    let (_directory, run_directory, found) = ended();
    assert_eq!(
        search::resume(&run_directory, &AtomicBool::new(false), None).unwrap(),
        found
    );
}

#[test]
fn resuming_a_search_that_ended_leaves_its_journal_unchanged() {
    let (_directory, run_directory, _found) = ended();
    let journal = fs::read(journal::path(&run_directory)).unwrap();
    search::resume(&run_directory, &AtomicBool::new(false), None).unwrap();
    assert_eq!(fs::read(journal::path(&run_directory)).unwrap(), journal);
}

#[test]
fn resuming_a_search_that_ended_copies_no_record_to_its_writer() {
    let (_directory, run_directory, _found) = ended();
    let mut written = Vec::new();
    search::resume(&run_directory, &AtomicBool::new(false), Some(&mut written)).unwrap();
    assert!(written.is_empty());
}

#[test]
fn resuming_a_search_that_ended_returns_it_even_when_asked_to_stop() {
    let (_directory, run_directory, found) = ended();
    assert_eq!(
        search::resume(&run_directory, &AtomicBool::new(true), None).unwrap(),
        found
    );
}
