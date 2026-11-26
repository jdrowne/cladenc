use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{DirectedEvolutionError, Found};
use cladenc::file_system::Disk;
use cladenc::journal::Settings;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const HOLDS_ITS_TESTS: &str = "COPY . /tests/\n";
const NEVER_BUILT: &str = "RUN exit 3\n";

struct Evolved {
    settings: Settings,
    found: Result<Found, DirectedEvolutionError>,
}

fn settings(directory: &Path, benchmark: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&Disk, benchmark).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
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

fn evolved_with_t2(task_toml: &str, steps_of_its_tests: Option<&str>) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]);
    fs::write(benchmark.join("t2/task.toml"), task_toml).unwrap();
    if let Some(steps) = steps_of_its_tests {
        fs::write(
            benchmark.join("t2/tests/Dockerfile"),
            format!("FROM {}\n{steps}", test_inputs::IMAGE),
        )
        .unwrap();
    }
    let settings = settings(directory.path(), &benchmark);
    let found = crate::real::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    Evolved { settings, found }
}

static WITH_A_VERIFIER_BUILT_FROM_ITS_TESTS: LazyLock<Evolved> =
    LazyLock::new(|| evolved_with_t2(SEPARATE, Some(HOLDS_ITS_TESTS)));

fn with_a_verifier_that_is_not_built() -> Evolved {
    evolved_with_t2(SEPARATE, Some(NEVER_BUILT))
}

fn with_a_verifier_in(image: &str) -> Evolved {
    evolved_with_t2(
        &format!("[verifier.environment]\ndocker_image = \"{image}\"\n"),
        None,
    )
}

fn environments_left_by(evolved: &Evolved) -> Vec<String> {
    let listed = Command::new("docker")
        .args(["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"])
        .arg(&evolved.settings.environment_image_repository)
        .output()
        .unwrap();
    String::from_utf8(listed.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_directed_evolution_whose_task_has_a_verifier_built_from_its_tests_runs_to_its_end() {
    assert!(WITH_A_VERIFIER_BUILT_FROM_ITS_TESTS.found.is_ok());
}

#[test]
fn a_directed_evolution_that_ended_leaves_no_verifier_environment_on_the_machine() {
    assert_eq!(
        environments_left_by(&WITH_A_VERIFIER_BUILT_FROM_ITS_TESTS),
        Vec::<String>::new()
    );
}

#[test]
fn a_verifier_whose_environment_is_not_built_leaves_no_environment_on_the_machine() {
    assert_eq!(
        environments_left_by(&with_a_verifier_that_is_not_built()),
        Vec::<String>::new()
    );
}

#[test]
fn a_verifier_whose_image_is_not_on_the_machine_stops_the_directed_evolution_naming_both() {
    assert!(matches!(
        with_a_verifier_in(NOT_ON_THE_MACHINE).found,
        Err(DirectedEvolutionError::VerifierNotOnTheMachine { task, image, .. })
            if task == "benchmark/t2" && image == NOT_ON_THE_MACHINE
    ));
}

#[test]
fn a_directed_evolution_whose_task_names_an_image_for_its_verifier_runs_to_its_end() {
    assert!(with_a_verifier_in(test_inputs::IMAGE).found.is_ok());
}

#[test]
fn a_directed_evolution_whose_verifier_runs_in_a_copy_of_an_environment_runs_to_its_end() {
    assert!(evolved_with_t2(SEPARATE, None).found.is_ok());
}

#[test]
fn a_directed_evolution_whose_task_and_verifier_ask_for_what_the_machine_has_runs_to_its_end() {
    assert!(
        evolved_with_t2(
            "[environment]\ncpus = 1\n\n[verifier.environment]\ncpus = 1\nmemory_mb = 64\n",
            None
        )
        .found
        .is_ok()
    );
}
