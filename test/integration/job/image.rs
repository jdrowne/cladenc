use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::job::{self, JobError};
use cladenc::workspace::Workspace;
use tempfile::TempDir;
use test_inputs::Image;

const MADE: &str = "RUN echo by the Dockerfile > /made-here\n";

struct Built {
    _directory: TempDir,
    workspace: Workspace,
    image: Image,
    result: Result<(), JobError>,
}

fn made() -> (TempDir, Workspace) {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let workspace = Workspace::create(&directory.path().join("run"), &progenitor).unwrap();
    (directory, workspace)
}

fn built_within(steps: &str, time_limit: u64, stop: bool) -> Built {
    let (directory, workspace) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), steps);
    let image = test_inputs::image();
    let stop = AtomicBool::new(stop);
    let result = job::build_image(&workspace, &dockerfile, &image.name, time_limit, &stop);
    Built {
        _directory: directory,
        workspace,
        image,
        result,
    }
}

fn built(steps: &str) -> Built {
    built_within(steps, 60, false)
}

fn printed_by_docker(built: &Built) -> String {
    ["stdout", "stderr"]
        .iter()
        .map(|file| fs::read_to_string(built.workspace.image().join(file)).unwrap())
        .collect()
}

#[test]
fn an_image_is_built_from_a_dockerfile() {
    assert!(built(MADE).result.is_ok());
}

#[test]
fn a_container_of_the_image_holds_what_the_dockerfile_made() {
    let built = built(MADE);
    assert_eq!(
        test_inputs::printed_in_a_container_of(&built.image.name, "cat /made-here"),
        "by the Dockerfile\n"
    );
}

#[test]
fn a_copy_of_the_dockerfile_is_kept_in_the_run_directory() {
    let built = built(MADE);
    assert_eq!(
        fs::read_to_string(built.workspace.image().join("Dockerfile")).unwrap(),
        format!("FROM {}\n{MADE}", test_inputs::IMAGE)
    );
}

#[test]
fn what_docker_printed_while_it_built_is_kept_in_the_run_directory() {
    assert!(printed_by_docker(&built(MADE)).contains("RUN echo by the Dockerfile"));
}

#[test]
fn a_dockerfile_with_a_step_that_fails_is_not_built() {
    assert!(matches!(
        built("RUN exit 3\n").result,
        Err(JobError::Exit(_))
    ));
}

#[test]
fn a_dockerfile_that_copies_a_file_is_not_built_since_it_is_given_no_file_to_copy() {
    assert!(matches!(
        built("COPY harness.sh /harness.sh\n").result,
        Err(JobError::Exit(_))
    ));
}

#[test]
fn a_build_that_runs_past_its_time_limit_is_ended() {
    assert!(matches!(
        built_within("RUN sleep 30\n", 1, false).result,
        Err(JobError::TimeLimit)
    ));
}

#[test]
fn a_build_is_ended_when_the_search_is_asked_to_stop() {
    assert!(matches!(
        built_within(MADE, 60, true).result,
        Err(JobError::Stopped)
    ));
}

#[test]
fn a_dockerfile_that_is_gone_is_not_built() {
    let (directory, workspace) = made();
    let image = test_inputs::image();
    let result = job::build_image(
        &workspace,
        &directory.path().join("missing"),
        &image.name,
        60,
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(JobError::Io(_))));
}
