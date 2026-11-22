use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::benchmark::Benchmark;
use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::journal::Settings;

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(directory, &["t1", "t2"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn started(settings: &Settings, outputs_directory: &Path) -> Result<Found, DirectedEvolutionError> {
    directed_evolution::start(
        settings,
        outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
}

fn with_a_file_of_the_key_that_holds_no_key(directory: &Path) -> Settings {
    Settings {
        model_key_file: Some(directory.join("model-key")),
        ..settings(directory)
    }
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_does_not_start() {
    let directory = test_inputs::temporary_directory();
    let settings = with_a_file_of_the_key_that_holds_no_key(directory.path());
    let found = started(&settings, &directory.path().join("outputs"));
    assert!(matches!(
        found,
        Err(DirectedEvolutionError::Key(file)) if file == directory.path().join("model-key")
    ));
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_holds_no_key_makes_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let settings = with_a_file_of_the_key_that_holds_no_key(directory.path());
    let _refused = started(&settings, &directory.path().join("outputs"));
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_directed_evolution_cannot_start_in_an_outputs_directory_that_exists() {
    let directory = test_inputs::temporary_directory();
    let found = started(&settings(directory.path()), directory.path());
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}

#[test]
fn a_directed_evolution_whose_benchmark_lacks_one_of_its_tasks_cannot_make_its_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    settings.benchmarks[0].tasks.push("missing".to_owned());
    let found = started(&settings, &directory.path().join("outputs"));
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}
