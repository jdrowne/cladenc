use std::fs;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use tempfile::TempDir;

use crate::benchmark::Benchmark;
use crate::containers::image;
use crate::containers::program::Ending;
use crate::containers::tests::double::Double;
use crate::containers::{ContainerError, Host};
use crate::outputs_directory::OutputsDirectory;

const TASK: &str = "benchmark/t1";
const NAME: &str = "cladenc-environments-0123456789abcdef:never-built";

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let benchmark = Benchmark::read(&test_inputs::benchmark(directory.path(), &["t1"])).unwrap();
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor).unwrap();
    outputs_directory.copy_benchmark(&benchmark).unwrap();
    (directory, outputs_directory)
}

fn with_a_file_where_the_environments_are_kept() -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = made();
    fs::write(outputs_directory.path().join("environments"), "").unwrap();
    (directory, outputs_directory)
}

#[test]
fn an_environment_whose_directory_in_the_outputs_directory_cannot_be_made_is_not_built() {
    let (_directory, outputs_directory) = with_a_file_where_the_environments_are_kept();
    let result = image::build_environment(
        Host {
            docker: &Double::default(),
            outputs_directory: &outputs_directory,
        },
        TASK,
        NAME,
        Duration::from_secs(600),
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn an_environment_of_a_grader_whose_directory_cannot_be_made_is_not_built() {
    let (_directory, outputs_directory) = with_a_file_where_the_environments_are_kept();
    let result = image::build_grader_environment(
        Host {
            docker: &Double::default(),
            outputs_directory: &outputs_directory,
        },
        TASK,
        NAME,
        Duration::from_secs(600),
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn a_dockerfile_that_is_gone_is_not_built() {
    let (directory, outputs_directory) = made();
    let result = image::build_image(
        Host {
            docker: &Double::default(),
            outputs_directory: &outputs_directory,
        },
        &directory.path().join("missing"),
        NAME,
        60,
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn a_build_that_exits_with_zero_has_built_its_image() {
    assert!(matches!(image::built(Ending::Exited(0)), Ok(())));
}

#[test]
fn a_build_that_exits_with_another_status_is_an_error_of_that_status() {
    assert!(matches!(
        image::built(Ending::Exited(2)),
        Err(ContainerError::Exit(2))
    ));
}

#[test]
fn a_build_that_runs_past_its_time_limit_is_an_error_of_the_time_limit() {
    assert!(matches!(
        image::built(Ending::TimedOut),
        Err(ContainerError::TimeLimit)
    ));
}
