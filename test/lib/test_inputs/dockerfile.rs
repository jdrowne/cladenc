use std::fs;
use std::process::Command;

use test_inputs::{IMAGE, dockerfile, image, printed_in_a_container_of, temporary_directory};

#[test]
fn a_dockerfile_is_the_file_named_dockerfile_in_the_directory() {
    let directory = temporary_directory();
    assert_eq!(
        dockerfile(directory.path(), ""),
        directory.path().join("Dockerfile")
    );
}

#[test]
fn a_dockerfile_starts_from_the_image_of_the_tests_and_goes_on_with_the_steps() {
    let directory = temporary_directory();
    let file = dockerfile(directory.path(), "RUN touch /made-here\n");
    assert_eq!(
        fs::read_to_string(file).unwrap(),
        format!("FROM {IMAGE}\nRUN touch /made-here\n")
    );
}

#[test]
fn a_dockerfile_of_no_step_starts_from_the_image_of_the_tests_and_does_no_more() {
    let directory = temporary_directory();
    let file = dockerfile(directory.path(), "");
    assert_eq!(fs::read_to_string(file).unwrap(), format!("FROM {IMAGE}\n"));
}

#[test]
fn an_image_built_from_a_dockerfile_holds_what_the_steps_made() {
    let directory = temporary_directory();
    let file = dockerfile(directory.path(), "RUN touch /made-here\n");
    let built = image();
    Command::new("docker")
        .args(["build", "--tag", &built.name, "--file"])
        .arg(file)
        .arg(directory.path())
        .output()
        .expect("docker can be called");
    assert_eq!(
        printed_in_a_container_of(&built.name, "ls /made-here"),
        "/made-here\n"
    );
}
