use std::fs;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Host, Program};
use cladenc::file_system::Disk;
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
    let benchmark =
        Benchmark::read(&Disk, &test_inputs::benchmark(directory.path(), &["t1"])).unwrap();
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    outputs_directory.copy_benchmark(&benchmark).unwrap();
    (directory, outputs_directory)
}

fn built(steps: &str) -> Built {
    let (directory, outputs_directory) = made();
    let environment = outputs_directory.task(TASK).join("environment");
    fs::write(environment.join("given.txt"), "from the build context\n").unwrap();
    fs::write(
        environment.join("Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let image = test_inputs::image();
    let result = containers::build_environment(
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

static BUILT: LazyLock<Built> = LazyLock::new(|| built(MADE));

static GRADER_BUILT: LazyLock<Built> = LazyLock::new(|| grader_built(HELD));

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
    assert!(BUILT.result.is_ok());
}

#[test]
fn a_container_of_the_environment_holds_what_the_dockerfile_of_the_task_made() {
    let built = &*BUILT;
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
    let built = &*BUILT;
    assert_eq!(
        label_of(&built.image.name, "cladenc.task"),
        "benchmark/t1\n"
    );
}

#[test]
fn the_image_of_an_environment_is_labelled_with_its_directed_evolution() {
    let built = &*BUILT;
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
    let other = built(MADE);
    assert_ne!(id_of(&BUILT), id_of(&other));
}

#[test]
fn what_docker_printed_while_it_built_the_environment_is_kept_in_the_outputs_directory() {
    assert!(printed_by_docker(&BUILT).contains("RUN echo by the environment"));
}

#[test]
fn an_environment_with_a_step_that_fails_is_not_built() {
    assert!(matches!(
        built("RUN exit 3\n").result,
        Err(ContainerError::Exit(_))
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
    assert!(GRADER_BUILT.result.is_ok());
}

#[test]
fn the_environment_of_a_grader_is_built_with_the_tests_of_its_task_as_its_build_context() {
    let built = &*GRADER_BUILT;
    assert_eq!(
        test_inputs::printed_in_a_container_of(&built.image.name, "cat /tests/notes.txt"),
        "from the tests of the task\n"
    );
}

#[test]
fn what_docker_printed_while_it_built_the_environment_of_a_grader_is_kept_in_grader() {
    assert!(printed_by_docker_for_the_grader(&GRADER_BUILT).contains("COPY notes.txt"));
}

#[test]
fn the_image_of_the_environment_of_a_grader_is_labelled_with_its_task() {
    let built = &*GRADER_BUILT;
    assert_eq!(
        label_of(&built.image.name, "cladenc.task"),
        "benchmark/t1\n"
    );
}

#[test]
fn the_image_of_the_environment_of_a_grader_is_labelled_with_its_directed_evolution() {
    let built = &*GRADER_BUILT;
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
