use std::sync::atomic::AtomicBool;
use std::time::Duration;

use tempfile::TempDir;

use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{self, TASK};
use crate::containers::{self, ContainerError, Machine};
use crate::outputs_directory::OutputsDirectory;

const IMAGE: &str = "an-image:made";
const BUILD: [&str; 1] = ["build"];
const REFUSED: Answer = Answer::Refused("docker is not there");
const UNSAID: &str = "Docker did not say how many CPUs and how much memory it has";

struct Built {
    _directory: TempDir,
    outputs_directory: OutputsDirectory,
    docker: Double,
    result: Result<(), ContainerError>,
}

impl Built {
    fn commands(&self) -> Vec<String> {
        self.docker.commands(&self.outputs_directory)
    }
}

fn image_built_by(docker: Double) -> Built {
    let (directory, outputs_directory) = given::made();
    let dockerfile = test_inputs::dockerfile(directory.path(), "");
    let result = containers::build_image(
        given::host(&docker, &outputs_directory),
        &dockerfile,
        IMAGE,
        30,
        &AtomicBool::new(false),
    );
    Built {
        _directory: directory,
        outputs_directory,
        docker,
        result,
    }
}

fn environment_built_by(docker: Double) -> Built {
    let (directory, outputs_directory) = given::made();
    let result = containers::build_environment(
        given::host(&docker, &outputs_directory),
        TASK,
        IMAGE,
        Duration::from_secs(30),
        &AtomicBool::new(false),
    );
    Built {
        _directory: directory,
        outputs_directory,
        docker,
        result,
    }
}

fn grader_environment_built_by(docker: Double) -> Built {
    let (directory, outputs_directory) = given::made();
    let result = containers::build_grader_environment(
        given::host(&docker, &outputs_directory),
        TASK,
        IMAGE,
        Duration::from_secs(30),
        &AtomicBool::new(false),
    );
    Built {
        _directory: directory,
        outputs_directory,
        docker,
        result,
    }
}

fn asked_of(docker: &Double) -> Vec<String> {
    docker
        .asked()
        .iter()
        .map(|asked| asked.arguments.join(" "))
        .collect()
}

#[test]
fn the_image_of_the_harnesses_is_built_from_a_copy_of_its_dockerfile_within_the_time_limit() {
    let built = image_built_by(Double::default());
    assert_eq!(
        built.commands(),
        [
            "timeout -s KILL 35 docker build --tag an-image:made --file \
             OUTPUTS/image/Dockerfile OUTPUTS/image/context"
        ]
    );
}

#[test]
fn what_the_build_of_the_image_of_the_harnesses_prints_is_kept_in_the_outputs_directory() {
    let built = image_built_by(Double::default());
    assert_eq!(
        built.docker.asked()[0].kept,
        built.outputs_directory.image()
    );
}

#[test]
fn a_build_of_the_image_of_the_harnesses_that_exits_with_zero_has_built_it() {
    assert!(image_built_by(Double::default()).result.is_ok());
}

#[test]
fn a_build_of_the_image_of_the_harnesses_that_exits_with_another_status_fails_with_it() {
    let docker = Double::default().answering(&BUILD, &[Answer::Exited(2)]);
    assert!(matches!(
        image_built_by(docker).result,
        Err(ContainerError::Exit(2))
    ));
}

#[test]
fn a_build_of_the_image_of_the_harnesses_that_runs_past_the_time_limit_fails_with_it() {
    let docker = Double::default().answering(&BUILD, &[Answer::TimedOut]);
    assert!(matches!(
        image_built_by(docker).result,
        Err(ContainerError::TimeLimit)
    ));
}

#[test]
fn a_build_of_the_image_of_the_harnesses_that_is_stopped_was_stopped() {
    let docker = Double::default().answering(&BUILD, &[Answer::Stopped]);
    assert!(matches!(
        image_built_by(docker).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn the_environment_of_a_task_is_built_from_its_directory_and_labelled_with_the_task() {
    let built = environment_built_by(Double::default());
    assert_eq!(
        built.commands(),
        [
            "docker build --tag an-image:made --label cladenc.task=benchmark/t1 --label \
             cladenc.directed-evolution=OUTPUTS OUTPUTS/benchmarks/benchmark/t1/environment"
        ]
    );
}

#[test]
fn what_the_build_of_an_environment_prints_is_kept_with_the_environment_of_the_task() {
    let built = environment_built_by(Double::default());
    assert_eq!(
        built.docker.asked()[0].kept,
        built.outputs_directory.environment(TASK)
    );
}

#[test]
fn a_build_of_an_environment_that_exits_with_zero_has_built_it() {
    assert!(environment_built_by(Double::default()).result.is_ok());
}

#[test]
fn a_build_of_an_environment_that_exits_with_another_status_fails_with_it() {
    let docker = Double::default().answering(&BUILD, &[Answer::Exited(1)]);
    assert!(matches!(
        environment_built_by(docker).result,
        Err(ContainerError::Exit(1))
    ));
}

#[test]
fn a_build_of_an_environment_that_runs_past_its_time_limit_fails_with_the_time_limit() {
    let docker = Double::default().answering(&BUILD, &[Answer::TimedOut]);
    assert!(matches!(
        environment_built_by(docker).result,
        Err(ContainerError::TimeLimit)
    ));
}

#[test]
fn a_build_of_an_environment_that_is_stopped_was_stopped() {
    let docker = Double::default().answering(&BUILD, &[Answer::Stopped]);
    assert!(matches!(
        environment_built_by(docker).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn the_environment_of_a_grader_is_built_from_the_tests_of_its_task() {
    let built = grader_environment_built_by(Double::default());
    assert_eq!(
        built.commands(),
        [
            "docker build --tag an-image:made --label cladenc.task=benchmark/t1 --label \
             cladenc.directed-evolution=OUTPUTS OUTPUTS/benchmarks/benchmark/t1/tests"
        ]
    );
}

#[test]
fn what_the_build_of_the_environment_of_a_grader_prints_is_kept_with_that_environment() {
    let built = grader_environment_built_by(Double::default());
    assert_eq!(
        built.docker.asked()[0].kept,
        built.outputs_directory.grader_environment(TASK)
    );
}

#[test]
fn a_build_of_the_environment_of_a_grader_that_exits_with_zero_has_built_it() {
    assert!(
        grader_environment_built_by(Double::default())
            .result
            .is_ok()
    );
}

#[test]
fn a_build_of_the_environment_of_a_grader_that_exits_with_another_status_fails_with_it() {
    let docker = Double::default().answering(&BUILD, &[Answer::Exited(1)]);
    assert!(matches!(
        grader_environment_built_by(docker).result,
        Err(ContainerError::Exit(1))
    ));
}

#[test]
fn docker_is_asked_how_many_cpus_and_how_much_memory_it_has() {
    let docker = Double::default();
    let _ = containers::machine(&docker);
    assert_eq!(
        asked_of(&docker),
        ["docker info --format {{.NCPU}} {{.MemTotal}}"]
    );
}

#[test]
fn the_memory_docker_has_is_counted_in_megabytes() {
    let docker = Double::default().answering(&["info"], &[Answer::Printed("8 16777216000\n")]);
    assert_eq!(
        containers::machine(&docker).ok(),
        Some(Machine {
            cpus: 8.0,
            memory_megabytes: 16000,
        })
    );
}

#[test]
fn what_docker_has_is_not_known_when_it_prints_something_other_than_two_numbers() {
    let docker = Double::default().answering(&["info"], &[Answer::Printed("many\n")]);
    assert_eq!(
        containers::machine(&docker).unwrap_err().to_string(),
        UNSAID
    );
}

#[test]
fn what_docker_has_is_not_known_when_it_prints_its_cpus_alone() {
    let docker = Double::default().answering(&["info"], &[Answer::Printed("8\n")]);
    assert_eq!(
        containers::machine(&docker).unwrap_err().to_string(),
        UNSAID
    );
}

#[test]
fn what_docker_has_is_not_known_when_docker_cannot_be_asked() {
    let docker = Double::default().answering(&["info"], &[REFUSED]);
    assert_eq!(
        containers::machine(&docker).unwrap_err().to_string(),
        "docker is not there"
    );
}

#[test]
fn the_images_to_remove_are_all_named_in_one_command() {
    let docker = Double::default();
    containers::remove_images(&docker, &["a:1".to_owned(), "b:2".to_owned()]);
    assert_eq!(asked_of(&docker), ["docker image rm --force a:1 b:2"]);
}

#[test]
fn an_image_is_looked_for_by_its_name() {
    let docker = Double::default();
    let _ = containers::find_image(&docker, IMAGE);
    assert_eq!(
        asked_of(&docker),
        ["docker image inspect --format {{.Id}} an-image:made"]
    );
}

#[test]
fn an_image_that_docker_shows_is_found() {
    assert!(containers::find_image(&Double::default(), IMAGE).is_ok());
}

#[test]
fn an_image_that_docker_does_not_show_is_not_found() {
    let docker = Double::default().answering(&["image", "inspect"], &[REFUSED]);
    assert!(containers::find_image(&docker, IMAGE).is_err());
}
