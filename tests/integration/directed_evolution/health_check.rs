use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{DirectedEvolutionError, Found};
use cladenc::file_system::Disk;
use cladenc::journal::{self, Settings, Started};
use tempfile::TempDir;

const NEVER_READY: &str = "[environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
const READY: &str = "[environment.healthcheck]\ncommand = \"true\"\n";

struct Evolved {
    directory: TempDir,
    found: Result<Found, DirectedEvolutionError>,
    said: String,
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
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 1,
        selection_tasks_percentage: 17.0,
        holdout_tasks: vec!["benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 17.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn with_a_task_that_is_never_ready() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]);
    fs::write(benchmark.join("t2/task.toml"), NEVER_READY).unwrap();
    let mut written = Vec::new();
    let found = crate::real::start(
        &settings(directory.path(), &benchmark),
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut written,
    );
    Evolved {
        directory,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn resumed_once_the_task_is_ready() -> Evolved {
    let mut evolved = with_a_task_that_is_never_ready();
    let outputs = evolved.directory.path().join("outputs");
    fs::write(outputs.join("benchmarks/benchmark/t2/task.toml"), READY).unwrap();
    let mut written = Vec::new();
    evolved.found = crate::real::resume(&outputs, &AtomicBool::new(false), &mut written);
    evolved.said = String::from_utf8(written).unwrap();
    evolved
}

fn outputs_of(evolved: &Evolved) -> String {
    evolved
        .directory
        .path()
        .join("outputs")
        .display()
        .to_string()
}

#[test]
fn a_task_that_is_never_ready_stops_the_directed_evolution_naming_the_attempt_and_the_tries() {
    assert!(matches!(
        with_a_task_that_is_never_ready().found,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { harness: 0, task },
            error: ContainerError::TaskHealthCheck { tries: 2, .. },
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_retry_of_the_health_check_of_a_task_is_said_with_its_attempt_its_wait_and_its_number() {
    assert!(with_a_task_that_is_never_ready().said.contains(
        "cladenc: the attempt of harness 0 on task benchmark/t2 is not carried out yet: the \
         health check of the task did not pass, and cladenc tries again in a new container \
         after 0 seconds, retry 1 of 1\n"
    ));
}

#[test]
fn a_harness_that_was_never_started_is_said_with_the_tries_and_where_the_health_check_is_kept() {
    let evolved = with_a_task_that_is_never_ready();
    assert!(evolved.said.contains(&format!(
        "cladenc: the attempt of harness 0 on task benchmark/t2 was not carried out: the health \
         check of the task did not pass in 2 tries, so the harness was not started, and \
         {}/running/attempt-0-benchmark+t2/health-check holds what the health check printed \
         the last time it ran\n",
        outputs_of(&evolved)
    )));
}

#[test]
fn an_attempt_whose_harness_was_never_started_is_not_said_to_be_one_it_did_not_complete() {
    assert!(
        !with_a_task_that_is_never_ready()
            .said
            .contains("did not complete task benchmark/t2")
    );
}

#[test]
fn an_attempt_whose_harness_was_never_started_is_recorded_as_started_and_not_as_finished() {
    let evolved = with_a_task_that_is_never_ready();
    let journal =
        fs::read_to_string(journal::path(&evolved.directory.path().join("outputs"))).unwrap();
    let records = [
        r#"{"record":"attempt_started","harness":0,"task":"benchmark/t2"}"#,
        r#"{"record":"attempt_finished","harness":0,"task":"benchmark/t2""#,
    ]
    .map(|record| journal.contains(record));
    assert_eq!(records, [true, false]);
}

#[test]
fn what_the_health_check_of_a_task_that_was_never_ready_printed_is_where_cladenc_says() {
    let evolved = with_a_task_that_is_never_ready();
    assert_eq!(
        fs::read_to_string(
            Path::new(&outputs_of(&evolved))
                .join("running/attempt-0-benchmark+t2/health-check/stdout")
        )
        .unwrap(),
        "not ready\n"
    );
}

#[test]
fn an_attempt_whose_harness_was_never_started_is_carried_out_again_when_resumed() {
    assert!(
        resumed_once_the_task_is_ready()
            .said
            .contains("cladenc: started again the attempt of harness 0 on task benchmark/t2\n")
    );
}

#[test]
fn a_directed_evolution_resumed_once_the_task_is_ready_runs_to_its_end() {
    assert!(resumed_once_the_task_is_ready().found.is_ok());
}
