use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, Host, Launch, Program, Trial};
use cladenc::file_system::Disk;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

const SAY_THE_CONTAINER_AND_STAY: &str = "hostname > id; sleep 300";
const SAY_THE_CONTAINER_OF_THE_TRIAL_AND_STAY: &str = "hostname; sleep 300";

type Trialled = (
    TempDir,
    OutputsDirectory,
    Result<containers::Trialled, ContainerError>,
);
type Diversified = (
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

fn trial(environment: &Environment) -> Trial<'_> {
    Trial {
        harness: 0,
        task: "benchmark/t1",
        environment,
    }
}

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn launch(run_command: &str) -> Launch<'_> {
    Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: MODEL,
        diversification_timeout: 60,
        max_verifier_health_check_retries: 0,
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

fn timed_out() -> Trialled {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_OF_THE_TRIAL_AND_STAY);
    let environment = Environment {
        image: test_inputs::IMAGE.to_owned(),
        configuration: TaskConfiguration {
            harness_time_limit: Some(Duration::from_secs(3)),
            ..TaskConfiguration::default()
        },
        verifier: None,
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        trial(&environment),
        &stop,
        &|_| (),
    );
    (directory, outputs_directory, result)
}

fn stopped_while_its_harness_runs() -> Trialled {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_OF_THE_TRIAL_AND_STAY);
    let id_file = outputs_directory
        .running_trial(0, "benchmark/t1")
        .join("stdout");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_trial(
            Host {
                docker: &Program,
                outputs_directory: &outputs_directory,
            },
            &launch,
            trial(&environment()),
            stop,
            &|_| (),
        )
    });
    (directory, outputs_directory, result)
}

fn stopped_while_its_verifier_runs() -> Trialled {
    let (directory, outputs_directory) = made();
    test_inputs::kept_verifier(
        &directory.path().join("outputs"),
        "t1",
        SAY_THE_CONTAINER_OF_THE_TRIAL_AND_STAY,
    );
    let launch = launch("true");
    let id_file = outputs_directory
        .running_trial(0, "benchmark/t1")
        .join("verifier/stdout");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_trial(
            Host {
                docker: &Program,
                outputs_directory: &outputs_directory,
            },
            &launch,
            trial(&environment()),
            stop,
            &|_| (),
        )
    });
    (directory, outputs_directory, result)
}

fn diversification_stopped_while_running() -> Diversified {
    let (directory, outputs_directory) = made();
    let launch = launch(SAY_THE_CONTAINER_AND_STAY);
    let id_file = outputs_directory
        .running_diversification(1)
        .join("child/id");
    let result = stopped_once_written(&id_file, |stop| {
        containers::run_diversification(
            Host {
                docker: &Program,
                outputs_directory: &outputs_directory,
            },
            &launch,
            1,
            0,
            stop,
        )
    });
    (directory, outputs_directory, result)
}

#[test]
fn the_time_limit_of_a_trial_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = timed_out();
    assert!(test_inputs::container_is_gone(
        &outputs_directory.trial(0, "benchmark/t1").join("stdout")
    ));
}

#[test]
fn a_trial_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_harness_runs();
    assert!(test_inputs::container_is_gone(
        &outputs_directory
            .running_trial(0, "benchmark/t1")
            .join("stdout")
    ));
}

#[test]
fn a_trial_stopped_while_its_verifier_runs_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = stopped_while_its_verifier_runs();
    assert!(test_inputs::container_is_gone(
        &outputs_directory
            .running_trial(0, "benchmark/t1")
            .join("verifier/stdout")
    ));
}

#[test]
fn a_diversification_whose_stop_flag_is_set_leaves_no_container_behind() {
    let (_directory, outputs_directory, _result) = diversification_stopped_while_running();
    assert!(test_inputs::container_is_gone(
        &outputs_directory
            .running_diversification(1)
            .join("child/id")
    ));
}
