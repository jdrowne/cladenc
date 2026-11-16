use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

struct Keyed {
    directory: TempDir,
    key_file: PathBuf,
    run_directory: PathBuf,
}

fn settings(directory: &Path, key_file: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: Some(key_file.to_owned()),
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "printf %s \"$CLADENC_MODEL_KEY\" > key-given".to_owned(),
        budget: 1,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        assessment_timeout: 30,
        seed: 7,
    }
}

fn evolved_with(key: Option<&str>, stop: bool) -> (Keyed, Result<Found, DirectedEvolutionError>) {
    let directory = test_inputs::temporary_directory();
    let key_file = directory.path().join("model-key");
    if let Some(key) = key {
        test_inputs::model_key_file(directory.path(), key);
    }
    let run_directory = directory.path().join("run");
    let settings = settings(directory.path(), &key_file);
    let found = directed_evolution::start(
        &settings,
        &run_directory,
        &AtomicBool::new(stop),
        &mut io::sink(),
    );
    let keyed = Keyed {
        directory,
        key_file,
        run_directory,
    };
    (keyed, found)
}

fn resumed(keyed: &Keyed) -> Result<Found, DirectedEvolutionError> {
    directed_evolution::resume(
        &keyed.run_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

fn key_given_to_the_first_child(keyed: &Keyed) -> String {
    fs::read_to_string(keyed.run_directory.join("harnesses/1/key-given")).unwrap()
}

#[test]
fn a_directed_evolution_hands_the_key_its_file_holds_to_its_harnesses() {
    let (keyed, _found) = evolved_with(Some("a-made-up-key\n"), false);
    assert_eq!(key_given_to_the_first_child(&keyed), "a-made-up-key");
}

#[test]
fn the_journal_of_a_directed_evolution_holds_the_file_of_the_key_and_not_the_key() {
    let (keyed, _found) = evolved_with(Some("a-made-up-key\n"), false);
    let journal = fs::read_to_string(journal::path(&keyed.run_directory)).unwrap();
    assert_eq!(
        (
            journal.contains("a-made-up-key"),
            journal.contains(keyed.key_file.to_str().unwrap())
        ),
        (false, true)
    );
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_does_not_start() {
    let (keyed, found) = evolved_with(None, false);
    assert!(matches!(found, Err(DirectedEvolutionError::Key(file)) if file == keyed.key_file));
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_makes_no_run_directory() {
    let (keyed, _found) = evolved_with(None, false);
    assert!(!keyed.run_directory.exists());
}

#[test]
fn a_resumed_directed_evolution_reads_the_key_afresh_from_its_file() {
    let (keyed, _stopped) = evolved_with(Some("a-made-up-key\n"), true);
    test_inputs::model_key_file(keyed.directory.path(), "a-key-that-replaced-it\n");
    let _ = resumed(&keyed);
    assert_eq!(
        key_given_to_the_first_child(&keyed),
        "a-key-that-replaced-it"
    );
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_is_gone_is_not_resumed() {
    let (keyed, _stopped) = evolved_with(Some("a-made-up-key\n"), true);
    fs::remove_file(&keyed.key_file).unwrap();
    assert!(
        matches!(resumed(&keyed), Err(DirectedEvolutionError::Key(file)) if file == keyed.key_file)
    );
}

#[test]
fn a_directed_evolution_that_is_not_resumed_for_its_key_starts_no_action() {
    let (keyed, _stopped) = evolved_with(Some("a-made-up-key\n"), true);
    let journal = fs::read(journal::path(&keyed.run_directory)).unwrap();
    fs::remove_file(&keyed.key_file).unwrap();
    let _ = resumed(&keyed);
    assert_eq!(
        fs::read(journal::path(&keyed.run_directory)).unwrap(),
        journal
    );
}

#[test]
fn a_directed_evolution_that_ended_is_resumed_without_its_key() {
    let (keyed, found) = evolved_with(Some("a-made-up-key\n"), false);
    fs::remove_file(&keyed.key_file).unwrap();
    assert_eq!(resumed(&keyed).ok(), Some(found.unwrap()));
}
