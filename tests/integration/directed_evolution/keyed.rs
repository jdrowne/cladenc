use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::Program;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

struct Keyed {
    directory: TempDir,
    key_file: PathBuf,
    outputs_directory: PathBuf,
}

fn settings(directory: &Path, key_file: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: Some(key_file.to_owned()),
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "[ -e end-with-125 ] && exit 125; printf %s \"$CLADENC_MODEL_KEY\" > \
                              key-given"
            .to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved_with(key: &str, stopped: bool) -> (Keyed, Result<Found, DirectedEvolutionError>) {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), key);
    let outputs_directory = directory.path().join("outputs");
    let settings = settings(directory.path(), &key_file);
    if stopped {
        test_inputs::end_with_125_on(directory.path(), "t1");
    }
    let found = directed_evolution::start(
        &Program,
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    if stopped {
        test_inputs::end_with_125_on_no_more(&outputs_directory, "t1");
    }
    let keyed = Keyed {
        directory,
        key_file,
        outputs_directory,
    };
    (keyed, found)
}

fn resumed(keyed: &Keyed) -> Result<Found, DirectedEvolutionError> {
    directed_evolution::resume(
        &Program,
        &keyed.outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

fn key_given_to_the_first_child(keyed: &Keyed) -> String {
    fs::read_to_string(keyed.outputs_directory.join("harnesses/1/key-given")).unwrap()
}

#[test]
fn a_directed_evolution_hands_the_key_its_file_holds_to_its_harnesses() {
    let (keyed, _found) = evolved_with("a-made-up-key\n", false);
    assert_eq!(key_given_to_the_first_child(&keyed), "a-made-up-key");
}

#[test]
fn the_journal_of_a_directed_evolution_holds_the_file_of_the_key_and_not_the_key() {
    let (keyed, _found) = evolved_with("a-made-up-key\n", false);
    let journal = fs::read_to_string(journal::path(&keyed.outputs_directory)).unwrap();
    assert_eq!(
        (
            journal.contains("a-made-up-key"),
            journal.contains(keyed.key_file.to_str().unwrap())
        ),
        (false, true)
    );
}

#[test]
fn a_resumed_directed_evolution_reads_the_key_afresh_from_its_file() {
    let (keyed, _stopped) = evolved_with("a-made-up-key\n", true);
    test_inputs::model_key_file(keyed.directory.path(), "a-key-that-replaced-it\n");
    let _ = resumed(&keyed);
    assert_eq!(
        key_given_to_the_first_child(&keyed),
        "a-key-that-replaced-it"
    );
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_is_gone_is_not_resumed() {
    let (keyed, _stopped) = evolved_with("a-made-up-key\n", true);
    fs::remove_file(&keyed.key_file).unwrap();
    assert!(
        matches!(resumed(&keyed), Err(DirectedEvolutionError::Key(file)) if file == keyed.key_file)
    );
}

#[test]
fn a_directed_evolution_that_is_not_resumed_for_its_key_starts_no_action() {
    let (keyed, _stopped) = evolved_with("a-made-up-key\n", true);
    let journal = fs::read(journal::path(&keyed.outputs_directory)).unwrap();
    fs::remove_file(&keyed.key_file).unwrap();
    let _ = resumed(&keyed);
    assert_eq!(
        fs::read(journal::path(&keyed.outputs_directory)).unwrap(),
        journal
    );
}

#[test]
fn a_directed_evolution_that_ended_is_resumed_without_its_key() {
    let (keyed, found) = evolved_with("a-made-up-key\n", false);
    fs::remove_file(&keyed.key_file).unwrap();
    assert_eq!(resumed(&keyed).ok(), Some(found.unwrap()));
}
