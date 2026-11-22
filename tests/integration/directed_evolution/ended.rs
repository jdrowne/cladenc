use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{self, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn ended() -> (TempDir, PathBuf, Found) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let found = directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
    .unwrap();
    (directory, outputs_directory, found)
}

#[test]
fn resuming_a_directed_evolution_that_ended_returns_what_the_directed_evolution_returned() {
    let (_directory, outputs_directory, found) = ended();
    assert_eq!(
        directed_evolution::resume(&outputs_directory, &AtomicBool::new(false), &mut io::sink())
            .unwrap(),
        found
    );
}

#[test]
fn resuming_a_directed_evolution_that_ended_leaves_its_journal_unchanged() {
    let (_directory, outputs_directory, _found) = ended();
    let journal = fs::read(journal::path(&outputs_directory)).unwrap();
    directed_evolution::resume(&outputs_directory, &AtomicBool::new(false), &mut io::sink())
        .unwrap();
    assert_eq!(
        fs::read(journal::path(&outputs_directory)).unwrap(),
        journal
    );
}

#[test]
fn resuming_a_directed_evolution_that_ended_says_that_it_had_finished_and_nothing_else() {
    let (_directory, outputs_directory, found) = ended();
    let mut written = Vec::new();
    directed_evolution::resume(&outputs_directory, &AtomicBool::new(false), &mut written).unwrap();
    assert_eq!(
        String::from_utf8(written).unwrap(),
        format!(
            "cladenc: the directed evolution had finished: its best harness is harness {}, kept \
             in {}\n",
            found.best_harness,
            found.directory.display()
        )
    );
}

#[test]
fn resuming_a_directed_evolution_that_ended_returns_it_even_when_asked_to_stop() {
    let (_directory, outputs_directory, found) = ended();
    assert_eq!(
        directed_evolution::resume(&outputs_directory, &AtomicBool::new(true), &mut io::sink())
            .unwrap(),
        found
    );
}
