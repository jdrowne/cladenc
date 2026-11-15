use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found};
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        budget: 10,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_procreations: 0,
        workers: 1,
        procreation_timeout: 30,
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
fn resuming_a_search_that_ended_says_that_it_had_finished_and_nothing_else() {
    let (_directory, run_directory, found) = ended();
    let mut written = Vec::new();
    search::resume(&run_directory, &AtomicBool::new(false), Some(&mut written)).unwrap();
    assert_eq!(
        String::from_utf8(written).unwrap(),
        format!(
            "cladenc: the search had finished: its best harness is harness {}, kept in {}\n",
            found.best_harness,
            found.directory.display()
        )
    );
}

#[test]
fn resuming_a_search_that_ended_returns_it_even_when_asked_to_stop() {
    let (_directory, run_directory, found) = ended();
    assert_eq!(
        search::resume(&run_directory, &AtomicBool::new(true), None).unwrap(),
        found
    );
}
