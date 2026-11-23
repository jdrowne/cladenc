use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::containers::tests::double::Double;
use crate::directed_evolution::tests::given::settings;
use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

fn started(
    directory: &Memory,
    settings: &Settings,
    outputs_directory: &Path,
) -> Result<Found, DirectedEvolutionError> {
    directed_evolution::start(
        Double::default().on(directory).outside(),
        settings,
        outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

fn with_a_file_of_the_key_that_holds_no_key(directory: &Memory) -> Settings {
    Settings {
        model_key_file: Some(directory.path().join("model-key")),
        ..settings(directory)
    }
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_does_not_start() {
    let directory = Memory::default();
    let settings = with_a_file_of_the_key_that_holds_no_key(&directory);
    let found = started(&directory, &settings, &directory.path().join("outputs"));
    assert!(matches!(
        found,
        Err(DirectedEvolutionError::Key(file)) if file == directory.path().join("model-key")
    ));
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_makes_no_outputs_directory() {
    let directory = Memory::default();
    let settings = with_a_file_of_the_key_that_holds_no_key(&directory);
    let _refused = started(&directory, &settings, &directory.path().join("outputs"));
    assert!(!directory.exists(directory.path().join("outputs")));
}

#[test]
fn a_directed_evolution_cannot_start_in_an_outputs_directory_that_exists() {
    let directory = Memory::default();
    let found = started(&directory, &settings(&directory), directory.path());
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}

#[test]
fn a_directed_evolution_whose_benchmark_lacks_one_of_its_tasks_cannot_make_its_outputs_directory() {
    let directory = Memory::default();
    let mut settings = settings(&directory);
    settings.benchmarks[0].tasks.push("missing".to_owned());
    let found = started(&directory, &settings, &directory.path().join("outputs"));
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}
