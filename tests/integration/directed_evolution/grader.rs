use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings, Started};
use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const HOLDS_ITS_TESTS: &str = "COPY . /tests/\n";
const NEVER_BUILT: &str = "RUN exit 3\n";
const NEVER_READY: &str = "[verifier.environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
const READY: &str = "[verifier.environment.healthcheck]\ncommand = \"true\"\n";

struct Evolved {
    directory: TempDir,
    settings: Settings,
    found: Result<Found, DirectedEvolutionError>,
    said: String,
}

fn settings(directory: &Path, benchmark: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(benchmark).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 17.0,
        holdout_tasks: vec!["benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 17.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved_with_t2(task_toml: &str, steps_of_its_tests: Option<&str>) -> Evolved {
    evolved_retrying(task_toml, steps_of_its_tests, 0)
}

fn evolved_retrying(
    task_toml: &str,
    steps_of_its_tests: Option<&str>,
    max_retries: u64,
) -> Evolved {
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
    let settings = Settings {
        max_grader_health_check_retries: max_retries,
        ..settings(directory.path(), &benchmark)
    };
    let mut written = Vec::new();
    let found = directed_evolution::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut written,
    );
    Evolved {
        directory,
        settings,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn with_a_grader_built_from_its_tests() -> Evolved {
    evolved_with_t2(SEPARATE, Some(HOLDS_ITS_TESTS))
}

fn with_a_grader_that_is_not_built() -> Evolved {
    evolved_with_t2(SEPARATE, Some(NEVER_BUILT))
}

fn with_a_grader_in(image: &str) -> Evolved {
    evolved_with_t2(
        &format!("[verifier.environment]\ndocker_image = \"{image}\"\n"),
        None,
    )
}

fn with_a_grader_that_is_never_ready() -> Evolved {
    evolved_retrying(NEVER_READY, None, 1)
}

fn resumed_once_the_grader_is_ready() -> Evolved {
    let mut evolved = with_a_grader_that_is_never_ready();
    let outputs = evolved.directory.path().join("outputs");
    fs::write(outputs.join("benchmarks/benchmark/t2/task.toml"), READY).unwrap();
    let mut written = Vec::new();
    evolved.found = directed_evolution::resume(&outputs, &AtomicBool::new(false), &mut written);
    evolved.said = String::from_utf8(written).unwrap();
    evolved
}

fn journal_of(evolved: &Evolved) -> String {
    fs::read_to_string(journal::path(&evolved.directory.path().join("outputs"))).unwrap()
}

fn outputs_of(evolved: &Evolved) -> String {
    evolved
        .directory
        .path()
        .join("outputs")
        .display()
        .to_string()
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
fn a_directed_evolution_says_that_it_builds_the_environment_of_a_grader_and_as_what_image() {
    let evolved = with_a_grader_built_from_its_tests();
    assert!(evolved.said.contains(&format!(
        "cladenc: building the environment of the grader of task benchmark/t2 as the image \
         {}:1-grader\n",
        evolved.settings.environment_image_repository
    )));
}

#[test]
fn the_environment_of_a_grader_is_built_after_that_of_its_task_and_before_that_of_the_next() {
    let evolved = with_a_grader_built_from_its_tests();
    let places = [
        "cladenc: building the environment of task benchmark/t2 ",
        "cladenc: building the environment of the grader of task benchmark/t2 ",
        "cladenc: building the environment of task benchmark/t3 ",
    ]
    .map(|start| {
        evolved
            .said
            .lines()
            .position(|line| line.starts_with(start))
    });
    assert_eq!(places, [Some(4), Some(5), Some(6)]);
}

#[test]
fn a_directed_evolution_whose_task_has_a_grader_built_from_its_tests_runs_to_its_end() {
    assert!(with_a_grader_built_from_its_tests().found.is_ok());
}

#[test]
fn what_docker_printed_while_it_built_the_environment_of_a_grader_is_kept_in_grader() {
    let evolved = with_a_grader_built_from_its_tests();
    assert!(
        Path::new(&outputs_of(&evolved))
            .join("environments/benchmark/t2/grader/stdout")
            .is_file()
    );
}

#[test]
fn a_directed_evolution_that_ended_leaves_no_environment_of_a_grader_on_the_machine() {
    assert_eq!(
        environments_left_by(&with_a_grader_built_from_its_tests()),
        Vec::<String>::new()
    );
}

#[test]
fn a_grader_whose_environment_is_not_built_stops_the_directed_evolution_naming_the_task() {
    assert!(matches!(
        with_a_grader_that_is_not_built().found,
        Err(DirectedEvolutionError::GraderEnvironment {
            task,
            error: ContainerError::Exit(_),
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_grader_whose_environment_is_not_built_is_said_with_how_its_build_ended() {
    assert!(
        with_a_grader_that_is_not_built()
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with(
                "cladenc: the build of the environment of the grader of task benchmark/t2 exited \
                 with "
            )
    );
}

#[test]
fn a_grader_whose_environment_is_not_built_is_said_with_where_what_docker_printed_is_kept() {
    let evolved = with_a_grader_that_is_not_built();
    let kept = format!(
        ": what Docker printed is kept in {}/environments/benchmark/t2/grader\n",
        outputs_of(&evolved)
    );
    assert!(evolved.said.ends_with(&kept));
}

#[test]
fn a_grader_whose_environment_is_not_built_leaves_no_environment_on_the_machine() {
    assert_eq!(
        environments_left_by(&with_a_grader_that_is_not_built()),
        Vec::<String>::new()
    );
}

#[test]
fn a_grader_whose_image_is_not_on_the_machine_stops_the_directed_evolution_naming_both() {
    assert!(matches!(
        with_a_grader_in(NOT_ON_THE_MACHINE).found,
        Err(DirectedEvolutionError::GraderNotOnTheMachine { task, image, .. })
            if task == "benchmark/t2" && image == NOT_ON_THE_MACHINE
    ));
}

#[test]
fn an_image_that_a_task_names_for_its_grader_is_said_to_be_taken() {
    assert!(with_a_grader_in(test_inputs::IMAGE).said.contains(
        "cladenc: taking the image debian:stable-slim as the environment of the grader of task \
         benchmark/t2, as its task.toml says\n"
    ));
}

#[test]
fn a_directed_evolution_whose_task_names_an_image_for_its_grader_runs_to_its_end() {
    assert!(with_a_grader_in(test_inputs::IMAGE).found.is_ok());
}

#[test]
fn a_grader_that_runs_in_a_copy_of_the_environment_of_its_task_has_no_environment_built() {
    assert!(
        !evolved_with_t2(SEPARATE, None)
            .said
            .contains("environment of the grader")
    );
}

#[test]
fn a_directed_evolution_whose_grader_runs_in_a_copy_of_an_environment_runs_to_its_end() {
    assert!(evolved_with_t2(SEPARATE, None).found.is_ok());
}

#[test]
fn a_grader_that_asks_for_more_cpus_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved_with_t2("[verifier.environment]\ncpus = 4096\n", None).found,
        Err(DirectedEvolutionError::TooLarge {
            task,
            key: "verifier.environment.cpus",
            ..
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_grader_that_asks_for_more_memory_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved_with_t2("[verifier.environment]\nmemory_mb = 1099511627776\n", None).found,
        Err(DirectedEvolutionError::TooLarge {
            task,
            key: "verifier.environment.memory_mb",
            ..
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_directed_evolution_whose_task_and_grader_ask_for_what_the_machine_has_runs_to_its_end() {
    assert!(
        evolved_with_t2(
            "[environment]\ncpus = 1\n\n[verifier.environment]\ncpus = 1\nmemory_mb = 64\n",
            None
        )
        .found
        .is_ok()
    );
}

#[test]
fn a_grader_that_is_never_ready_stops_the_directed_evolution_naming_the_attempt_and_the_tries() {
    assert!(matches!(
        with_a_grader_that_is_never_ready().found,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { harness: 0, task },
            error: ContainerError::GraderHealthCheck { tries: 2, .. },
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_retry_of_the_health_check_of_a_grader_is_said_with_its_attempt_its_wait_and_its_number() {
    assert!(with_a_grader_that_is_never_ready().said.contains(
        "cladenc: the attempt of harness 0 on task benchmark/t2 is not graded yet: the health \
         check of the container of its grader did not pass, and cladenc tries again in a new \
         container after 0 seconds, retry 1 of 1\n"
    ));
}

#[test]
fn a_grader_that_was_never_run_is_said_with_the_tries_and_where_its_health_check_is_kept() {
    let evolved = with_a_grader_that_is_never_ready();
    assert!(evolved.said.contains(&format!(
        "cladenc: the attempt of harness 0 on task benchmark/t2 was not graded: the health \
         check of the container of its grader did not pass in 2 tries, so the grader was not \
         run, and {}/running/attempt-0-benchmark+t2/grader/health-check holds what the health \
         check printed the last time it ran\n",
        outputs_of(&evolved)
    )));
}

#[test]
fn an_attempt_whose_grader_was_never_run_is_not_said_to_be_one_the_harness_did_not_complete() {
    assert!(
        !with_a_grader_that_is_never_ready()
            .said
            .contains("did not complete task benchmark/t2")
    );
}

#[test]
fn an_attempt_whose_grader_was_never_run_is_recorded_as_started_and_not_as_finished() {
    let journal = journal_of(&with_a_grader_that_is_never_ready());
    let records = [
        r#"{"record":"attempt_started","harness":0,"task":"benchmark/t2"}"#,
        r#"{"record":"attempt_finished","harness":0,"task":"benchmark/t2""#,
    ]
    .map(|record| journal.contains(record));
    assert_eq!(records, [true, false]);
}

#[test]
fn what_the_health_check_of_a_grader_that_was_never_run_printed_is_where_cladenc_says() {
    let evolved = with_a_grader_that_is_never_ready();
    assert_eq!(
        fs::read_to_string(
            Path::new(&outputs_of(&evolved))
                .join("running/attempt-0-benchmark+t2/grader/health-check/stdout")
        )
        .unwrap(),
        "not ready\n"
    );
}

#[test]
fn an_attempt_whose_grader_was_never_run_is_carried_out_again_when_resumed() {
    assert!(
        resumed_once_the_grader_is_ready()
            .said
            .contains("cladenc: started again the attempt of harness 0 on task benchmark/t2\n")
    );
}

#[test]
fn a_directed_evolution_resumed_once_the_grader_is_ready_runs_to_its_end() {
    assert!(resumed_once_the_grader_is_ready().found.is_ok());
}
