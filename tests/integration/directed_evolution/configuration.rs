use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{DirectedEvolutionError, Found};
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const SAY_THE_MODE: &str = "echo \"$MODE\"";

struct Evolved {
    directory: TempDir,
    found: Result<Found, DirectedEvolutionError>,
}

fn in_the_image(image: &str) -> String {
    format!("[environment]\ndocker_image = \"{image}\"\n")
}

fn settings(directory: &Path, task_tomls: &[(&str, &str)]) -> Settings {
    let benchmark = test_inputs::benchmark(directory, &["t1", "t2", "t3"]);
    for (task, task_toml) in task_tomls {
        fs::write(benchmark.join(task).join("task.toml"), task_toml).unwrap();
    }
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&Disk, &benchmark).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: SAY_THE_MODE.to_owned(),
        max_diversifications: 1,
        max_concurrent_containers: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 17.0,
        holdout_tasks: vec!["benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 17.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved(task_tomls: &[(&str, &str)]) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), task_tomls);
    let found = crate::real::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    Evolved { directory, found }
}

static WITH_A_MODE_FOR_T1: LazyLock<Evolved> =
    LazyLock::new(|| evolved(&[("t1", "[environment.env]\nMODE = \"strict\"\n")]));

fn with_t2_in(image: &str) -> Evolved {
    evolved(&[("t2", &in_the_image(image))])
}

fn outputs_of(evolved: &Evolved) -> PathBuf {
    evolved.directory.path().join("outputs")
}

fn printed_in_the_trial_at(evolved: &Evolved, task: &str) -> String {
    let trial = test_inputs::kept_trial(&outputs_of(evolved), 0, task);
    fs::read_to_string(trial.join("stdout")).unwrap()
}

#[test]
fn a_directed_evolution_in_which_every_task_names_an_image_runs_to_its_end() {
    let image = in_the_image(test_inputs::IMAGE);
    assert!(
        evolved(&[("t1", &image), ("t2", &image), ("t3", &image)])
            .found
            .is_ok()
    );
}

#[test]
fn a_trial_on_a_task_that_names_an_image_is_carried_out_in_a_container_of_that_image() {
    let evolved = with_t2_in(test_inputs::IMAGE);
    let trial = test_inputs::kept_trial(&outputs_of(&evolved), 0, "benchmark/t2");
    assert!(!trial.join("work").exists());
}

#[test]
fn a_trial_on_a_task_is_given_the_variables_its_task_toml_names() {
    assert_eq!(
        printed_in_the_trial_at(&WITH_A_MODE_FOR_T1, "benchmark/t1"),
        "strict\n"
    );
}

#[test]
fn a_trial_on_a_task_is_given_no_variable_that_another_task_names() {
    assert_eq!(
        printed_in_the_trial_at(&WITH_A_MODE_FOR_T1, "benchmark/t2"),
        "\n"
    );
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_stops_the_directed_evolution_naming_both() {
    assert!(matches!(
        with_t2_in(NOT_ON_THE_MACHINE).found,
        Err(DirectedEvolutionError::NotOnTheMachine { task, image, .. })
            if task == "benchmark/t2" && image == NOT_ON_THE_MACHINE
    ));
}

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved(&[("t2", "[environment]\ncpus = 100000\n")]).found,
        Err(DirectedEvolutionError::TooLarge { task, key: "cpus", .. }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_task_that_asks_for_more_memory_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved(&[("t2", "[environment]\nmemory_mb = 100000000\n")]).found,
        Err(DirectedEvolutionError::TooLarge { task, key: "memory_mb", .. })
            if task == "benchmark/t2"
    ));
}

#[test]
fn a_directed_evolution_whose_tasks_ask_for_what_the_machine_has_runs_to_its_end() {
    let within = "[environment]\ncpus = 1\nmemory_mb = 512\n";
    assert!(
        evolved(&[("t1", within), ("t2", within), ("t3", within)])
            .found
            .is_ok()
    );
}
