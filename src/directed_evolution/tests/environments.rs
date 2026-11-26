use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::environments::{self, Making, Whose};
use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::tests::double::Double;
use crate::sandboxes::{Host, SandboxError};

const TASK: &str = "benchmark/t1";

fn not_built(whose: Whose) -> (DirectedEvolutionError, String) {
    let outputs_directory = OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"));
    let stop = AtomicBool::new(false);
    let mut written: Vec<u8> = Vec::new();
    let docker = Double::default();
    let mut making = Making {
        host: Host {
            docker: &docker,
            outputs_directory: &outputs_directory,
        },
        stop: &stop,
        writer: &mut written,
        machine: None,
    };
    let error = making.not_built(whose, TASK, SandboxError::Exit(1));
    (error, String::from_utf8(written).unwrap())
}

#[test]
fn an_environment_that_is_not_built_is_an_error_of_the_environment_of_its_task() {
    assert!(matches!(
        not_built(Whose::TheTask).0,
        DirectedEvolutionError::Environment {
            task,
            error: SandboxError::Exit(1)
        } if task == TASK
    ));
}

#[test]
fn a_verifier_environment_that_is_not_built_is_an_error_of_the_verifier_of_its_task() {
    assert!(matches!(
        not_built(Whose::TheVerifier).0,
        DirectedEvolutionError::VerifierEnvironment {
            task,
            error: SandboxError::Exit(1)
        } if task == TASK
    ));
}

#[test]
fn an_environment_that_is_not_built_is_said_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        not_built(Whose::TheTask).1,
        "cladenc: the build of the environment of task benchmark/t1 exited with status 1: what \
         Docker printed is kept in /outputs/environments/benchmark/t1\n"
    );
}

#[test]
fn a_verifier_environment_that_is_not_built_is_said_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        not_built(Whose::TheVerifier).1,
        "cladenc: the build of the verifier environment of task benchmark/t1 exited with \
         status 1: what Docker printed is kept in /outputs/environments/benchmark/t1/verifier\n"
    );
}

#[test]
fn the_memory_a_task_asks_for_is_compared_with_that_of_the_machine_as_megabytes() {
    assert_eq!(environments::megabytes(Some(4096)), Some(4096.0));
}

#[test]
fn a_task_that_asks_for_no_memory_asks_for_no_megabytes() {
    assert_eq!(environments::megabytes(None), None);
}

#[test]
fn the_verifier_image_of_a_task_is_named_after_the_image_of_the_task() {
    assert_eq!(
        environments::of_the_verifier("cladenc-environments-0123456789abcdef:t1"),
        "cladenc-environments-0123456789abcdef:t1-verifier"
    );
}
