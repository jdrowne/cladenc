use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, Attempt, ContainerError, Environment, Launch};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

const SAY_THE_CONTAINER_AND_STAY: &str = "hostname > id; sleep 300";
const SAY_THE_CONTAINER_OF_THE_ATTEMPT_AND_STAY: &str = "hostname; sleep 300";

type Attempted = (
    TempDir,
    OutputsDirectory,
    Result<containers::Attempted, ContainerError>,
);
type Procreated = (
    TempDir,
    OutputsDirectory,
    Result<containers::Creation, ContainerError>,
);

fn environment() -> Environment {
    Environment {
        image: test_inputs::IMAGE.to_owned(),
        ..Environment::default()
    }
}

fn attempt(environment: &Environment) -> Attempt<'_> {
    Attempt {
        harness: 0,
        task: "benchmark/t1",
        environment,
    }
}

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn launch(run_command: &str) -> Launch<'_> {
    Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: MODEL,
        procreation_timeout: 60,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
    }
}

fn stopped_once_written<Outcome>(
    written: &Path,
    run: impl FnOnce(&AtomicBool) -> Result<Outcome, ContainerError>,
) -> Result<Outcome, ContainerError> {
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(written));
            stop.store(true, Ordering::SeqCst);
        });
        run(&stop)
    })
}

fn timed_out() -> Attempted {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_OF_THE_ATTEMPT_AND_STAY);
    let environment = Environment {
        image: test_inputs::IMAGE.to_owned(),
        configuration: TaskConfiguration {
            harness_time_limit: Some(Duration::from_secs(3)),
            ..TaskConfiguration::default()
        },
        grader: None,
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_attempt(
        &outputs_directory,
        &launch,
        attempt(&environment),
        &stop,
        &|_| (),
    );
    (directory, outputs_directory, result)
}

fn stopped_while_its_harness_runs() -> Attempted {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_OF_THE_ATTEMPT_AND_STAY);
    let id_file = outputs_directory
        .running_attempt(0, "benchmark/t1")
        .join("stdout");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_attempt(
            &outputs_directory,
            &launch,
            attempt(&environment()),
            stop,
            &|_| (),
        )
    });
    (directory, outputs_directory, result)
}

fn stopped_while_its_grader_runs() -> Attempted {
    let (directory, outputs_directory) = made();
    test_inputs::kept_grader(
        &directory.path().join("outputs"),
        "t1",
        SAY_THE_CONTAINER_OF_THE_ATTEMPT_AND_STAY,
    );
    let launch = launch("true");
    let id_file = outputs_directory
        .running_attempt(0, "benchmark/t1")
        .join("grader/stdout");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_attempt(
            &outputs_directory,
            &launch,
            attempt(&environment()),
            stop,
            &|_| (),
        )
    });
    (directory, outputs_directory, result)
}

fn procreation_stopped_while_running() -> Procreated {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_AND_STAY);
    let id_file = outputs_directory.running_procreation(1).join("child/id");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_procreation(&outputs_directory, &launch, 1, 0, stop)
    });
    (directory, outputs_directory, result)
}

#[test]
fn the_time_limit_of_an_attempt_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = timed_out();
    assert!(test_inputs::container_is_gone(
        &outputs_directory.attempt(0, "benchmark/t1").join("stdout")
    ));
}

#[test]
fn an_attempt_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _outputs_directory, result) = stopped_while_its_harness_runs();
    assert!(matches!(result, Err(ContainerError::Stopped)));
}

#[test]
fn an_attempt_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_harness_runs();
    assert!(test_inputs::container_is_gone(
        &outputs_directory
            .running_attempt(0, "benchmark/t1")
            .join("stdout")
    ));
}

#[test]
fn an_attempt_whose_stop_flag_is_set_leaves_its_directory_under_running() {
    let (_directory, outputs_directory, _result) = stopped_while_its_harness_runs();
    assert!(
        outputs_directory
            .running_attempt(0, "benchmark/t1")
            .join("specification")
            .is_file()
    );
}

#[test]
fn an_attempt_stopped_while_its_grader_runs_reports_that_it_was_stopped() {
    let (_directory, _outputs_directory, result) = stopped_while_its_grader_runs();
    assert!(matches!(result, Err(ContainerError::Stopped)));
}

#[test]
fn an_attempt_stopped_while_its_grader_runs_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_grader_runs();
    assert!(test_inputs::container_is_gone(
        &outputs_directory
            .running_attempt(0, "benchmark/t1")
            .join("grader/stdout")
    ));
}

#[test]
fn an_attempt_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, outputs_directory) = made();
    let stop = AtomicBool::new(true);
    assert!(matches!(
        containers::run_attempt(
            &outputs_directory,
            &launch("sleep 300"),
            attempt(&environment()),
            &stop,
            &|_| ()
        ),
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn a_procreation_whose_stop_flag_was_set_before_it_started_is_stopped() {
    let (_directory, outputs_directory) = made();
    let stop = AtomicBool::new(true);
    assert!(matches!(
        containers::run_procreation(&outputs_directory, &launch("sleep 300"), 1, 0, &stop),
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn a_procreation_whose_stop_flag_is_set_reports_that_it_was_stopped() {
    let (_directory, _outputs_directory, result) = procreation_stopped_while_running();
    assert!(matches!(result, Err(ContainerError::Stopped)));
}

#[test]
fn a_procreation_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = procreation_stopped_while_running();
    assert!(test_inputs::container_is_gone(
        &outputs_directory.running_procreation(1).join("child/id")
    ));
}
