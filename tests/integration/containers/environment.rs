use std::fs;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError};
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";

const MADE: &str = "RUN echo by the environment > /made-here\n";

struct Built {
    _directory: TempDir,
    outputs_directory: OutputsDirectory,
    image: Image,
    result: Result<(), ContainerError>,
}

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let benchmark = Benchmark::read(&test_inputs::benchmark(directory.path(), &["t1"])).unwrap();
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    outputs_directory.copy_benchmark(&benchmark).unwrap();
    (directory, outputs_directory)
}

fn built_from(steps: &str, stop: bool) -> Built {
    built_within(600, steps, stop)
}

fn built_within(time_limit: u64, steps: &str, stop: bool) -> Built {
    let (directory, outputs_directory) = made();
    let environment = outputs_directory.task(TASK).join("environment");
    fs::write(environment.join("given.txt"), "from the build context\n").unwrap();
    fs::write(
        environment.join("Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let image = test_inputs::image();
    let stop = AtomicBool::new(stop);
    let result = containers::build_environment(
        &outputs_directory,
        TASK,
        &image.name,
        Duration::from_secs(time_limit),
        &stop,
    );
    Built {
        _directory: directory,
        outputs_directory,
        image,
        result,
    }
}

fn built(steps: &str) -> Built {
    built_from(steps, false)
}

fn printed_by_docker(built: &Built) -> String {
    ["stdout", "stderr"]
        .iter()
        .map(|file| {
            fs::read_to_string(built.outputs_directory.environment(TASK).join(file)).unwrap()
        })
        .collect()
}

fn label_of(image: &str) -> String {
    let inspected = Command::new("docker")
        .args(["image", "inspect", "--format"])
        .arg(r#"{{index .Config.Labels "cladenc.task"}}"#)
        .arg(image)
        .output()
        .unwrap();
    String::from_utf8(inspected.stdout).unwrap()
}

#[test]
fn the_environment_of_a_task_is_built_from_its_dockerfile() {
    assert!(built(MADE).result.is_ok());
}

#[test]
fn a_container_of_the_environment_holds_what_the_dockerfile_of_the_task_made() {
    let built = built(MADE);
    assert_eq!(
        test_inputs::printed_in_a_container_of(&built.image.name, "cat /made-here"),
        "by the environment\n"
    );
}

#[test]
fn the_environment_is_built_with_the_environment_directory_of_the_task_as_its_build_context() {
    let built = built("COPY given.txt /given.txt\n");
    assert_eq!(
        test_inputs::printed_in_a_container_of(&built.image.name, "cat /given.txt"),
        "from the build context\n"
    );
}

#[test]
fn the_image_of_an_environment_is_labelled_with_its_task() {
    let built = built(MADE);
    assert_eq!(label_of(&built.image.name), "benchmark/t1\n");
}

#[test]
fn what_docker_printed_while_it_built_the_environment_is_kept_in_the_outputs_directory() {
    assert!(printed_by_docker(&built(MADE)).contains("RUN echo by the environment"));
}

#[test]
fn an_environment_with_a_step_that_fails_is_not_built() {
    assert!(matches!(
        built("RUN exit 3\n").result,
        Err(ContainerError::Exit(_))
    ));
}

#[test]
fn the_build_of_an_environment_is_ended_when_the_directed_evolution_is_asked_to_stop() {
    assert!(matches!(
        built_from(MADE, true).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn an_environment_whose_directory_in_the_outputs_directory_cannot_be_made_is_not_built() {
    let (_directory, outputs_directory) = made();
    fs::write(outputs_directory.path().join("environments"), "").unwrap();
    let image = test_inputs::image();
    let result = containers::build_environment(
        &outputs_directory,
        TASK,
        &image.name,
        Duration::from_secs(600),
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn the_build_of_an_environment_that_runs_past_its_time_limit_is_ended() {
    assert!(matches!(
        built_within(2, "RUN sleep 300\n", false).result,
        Err(ContainerError::TimeLimit)
    ));
}
