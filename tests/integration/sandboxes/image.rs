use std::fs;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Host, Program, SandboxError};
use tempfile::TempDir;
use test_inputs::Image;

const MADE: &str = "RUN echo by the Dockerfile > /made-here\n";

struct Built {
    _directory: TempDir,
    outputs_directory: OutputsDirectory,
    image: Image,
    result: Result<(), SandboxError>,
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
    (directory, outputs_directory)
}

fn built(steps: &str) -> Built {
    let (directory, outputs_directory) = made();
    let dockerfile = test_inputs::dockerfile(directory.path(), steps);
    let image = test_inputs::image();
    let result = sandboxes::build_image(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &dockerfile,
        &image.name,
        60,
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

fn printed_by_docker(built: &Built) -> String {
    ["stdout", "stderr"]
        .iter()
        .map(|file| fs::read_to_string(built.outputs_directory.image().join(file)).unwrap())
        .collect()
}

#[test]
fn an_image_is_built_from_a_dockerfile() {
    assert!(BUILT.result.is_ok());
}

#[test]
fn a_sandbox_of_the_image_holds_what_the_dockerfile_made() {
    let built = &*BUILT;
    assert_eq!(
        test_inputs::printed_in_a_sandbox_of(&built.image.name, "cat /made-here"),
        "by the Dockerfile\n"
    );
}

#[test]
fn what_docker_printed_while_it_built_is_kept_in_the_outputs_directory() {
    assert!(printed_by_docker(&BUILT).contains("RUN echo by the Dockerfile"));
}

#[test]
fn a_dockerfile_that_copies_a_file_is_not_built_since_it_is_given_no_file_to_copy() {
    assert!(matches!(
        built("COPY harness.sh /harness.sh\n").result,
        Err(SandboxError::Exit(_))
    ));
}
