use std::fs;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Host, Program};
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";

const MADE: &str = "RUN echo by the environment > /made-here\n";
const HELD: &str = "COPY notes.txt /tests/notes.txt\n";

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
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
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

fn inspected(image: &str, format: &str) -> String {
    let inspected = Command::new("docker")
        .args(["image", "inspect", "--format", format, image])
        .output()
        .unwrap();
    String::from_utf8(inspected.stdout).unwrap()
}

fn label_of(image: &str, label: &str) -> String {
    inspected(image, &format!("{{{{index .Config.Labels \"{label}\"}}}}"))
}

fn id_of(built: &Built) -> String {
    inspected(&built.image.name, "{{.Id}}")
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
    assert_eq!(
        label_of(&built.image.name, "cladenc.task"),
        "benchmark/t1\n"
    );
}

#[test]
fn the_image_of_an_environment_is_labelled_with_its_directed_evolution() {
    let built = built(MADE);
    assert_eq!(
        label_of(&built.image.name, "cladenc.directed-evolution"),
        format!(
            "{}\n",
            fs::canonicalize(built.outputs_directory.path())
                .unwrap()
                .display()
        )
    );
}

#[test]
fn the_environments_two_directed_evolutions_build_from_one_task_are_two_images() {
    let (one, other) = (built(MADE), built(MADE));
    assert_ne!(id_of(&one), id_of(&other));
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
fn the_build_of_an_environment_that_runs_past_its_time_limit_is_ended() {
    assert!(matches!(
        built_within(2, "RUN sleep 300\n", false).result,
        Err(ContainerError::TimeLimit)
    ));
}

fn grader_built(steps: &str) -> Built {
    let (directory, outputs_directory) = made();
    let tests = outputs_directory.task(TASK).join("tests");
    fs::write(tests.join("notes.txt"), "from the tests of the task\n").unwrap();
    fs::write(
        tests.join("Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let image = test_inputs::image();
    let result = containers::build_grader_environment(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        TASK,
        &image.name,
        Duration::from_secs(600),
        &AtomicBool::new(false),
    );
    Built {
        _directory: directory,
        outputs_directory,
        image,
        result,
    }
}

fn printed_by_docker_for_the_grader(built: &Built) -> String {
    ["stdout", "stderr"]
        .iter()
        .map(|file| {
            fs::read_to_string(built.outputs_directory.grader_environment(TASK).join(file)).unwrap()
        })
        .collect()
}

#[test]
fn the_environment_of_a_grader_is_built_from_the_dockerfile_in_the_tests_of_its_task() {
    assert!(grader_built(HELD).result.is_ok());
}

#[test]
fn the_environment_of_a_grader_is_built_with_the_tests_of_its_task_as_its_build_context() {
    let built = grader_built(HELD);
    assert_eq!(
        test_inputs::printed_in_a_container_of(&built.image.name, "cat /tests/notes.txt"),
        "from the tests of the task\n"
    );
}

#[test]
fn what_docker_printed_while_it_built_the_environment_of_a_grader_is_kept_in_grader() {
    assert!(printed_by_docker_for_the_grader(&grader_built(HELD)).contains("COPY notes.txt"));
}

#[test]
fn the_image_of_the_environment_of_a_grader_is_labelled_with_its_task() {
    let built = grader_built(HELD);
    assert_eq!(
        label_of(&built.image.name, "cladenc.task"),
        "benchmark/t1\n"
    );
}

#[test]
fn the_image_of_the_environment_of_a_grader_is_labelled_with_its_directed_evolution() {
    let built = grader_built(HELD);
    assert_eq!(
        label_of(&built.image.name, "cladenc.directed-evolution"),
        format!(
            "{}\n",
            fs::canonicalize(built.outputs_directory.path())
                .unwrap()
                .display()
        )
    );
}

#[test]
fn an_environment_of_a_grader_with_a_step_that_fails_is_not_built() {
    assert!(matches!(
        grader_built("RUN exit 3\n").result,
        Err(ContainerError::Exit(_))
    ));
}
