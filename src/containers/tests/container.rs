use std::sync::atomic::AtomicBool;

use crate::containers::ContainerError;
use crate::containers::container::Up;
use crate::containers::program::Limits;
use crate::outputs_directory::OutputsDirectory;

#[test]
fn a_container_with_no_directory_to_keep_what_docker_says_is_not_started() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let started = Up::start(
        &OutputsDirectory::at(directory.path()),
        test_inputs::IMAGE,
        &directory.path().join("missing"),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(started, Err(ContainerError::Io(_))));
}
