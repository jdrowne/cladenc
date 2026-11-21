use std::sync::atomic::AtomicBool;

use crate::containers::container::{self, Up};
use crate::containers::program::Limits;
use crate::containers::{ContainerError, Environment};
use crate::outputs_directory::OutputsDirectory;

#[test]
fn a_container_with_no_directory_to_keep_what_docker_says_is_not_started() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let environment = Environment {
        image: test_inputs::IMAGE.to_owned(),
        ..Environment::default()
    };
    let started = Up::start(
        &OutputsDirectory::at(directory.path()),
        &environment,
        &directory.path().join("missing"),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(started, Err(ContainerError::Io(_))));
}

#[test]
fn the_variables_of_a_container_are_written_for_docker_one_to_a_line_as_a_name_and_a_value() {
    assert_eq!(
        container::file_of(&[
            ("MODE".to_owned(), "strict".to_owned()),
            ("NOTE".to_owned(), "a = b".to_owned())
        ]),
        "MODE=strict\nNOTE=a = b\n"
    );
}

#[test]
fn a_container_with_no_variable_is_given_none() {
    assert_eq!(container::file_of(&[]), "");
}
