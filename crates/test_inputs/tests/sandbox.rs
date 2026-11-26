use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use test_inputs::{
    IMAGE, printed_in_a_sandbox_of, sandbox_is_gone, sandbox_is_running, temporary_directory,
};

const REMOVED_ONCE_ENDED: &str = "--rm=true";
const KEPT_ONCE_ENDED: &str = "--rm=false";

fn docker(arguments: &[&str]) -> String {
    let ran = Command::new("docker")
        .args(arguments)
        .output()
        .expect("docker can be called");
    String::from_utf8(ran.stdout).expect("what docker printed is text")
}

fn id_in(id_file: &Path) -> String {
    fs::read_to_string(id_file).unwrap().trim().to_owned()
}

fn started(directory: &Path, removal: &str, command: &str) -> PathBuf {
    let id_file = directory.join("id");
    let id = docker(&[
        "run", "--detach", "--pull", "never", removal, IMAGE, "sh", "-c", command,
    ]);
    fs::write(&id_file, id).unwrap();
    id_file
}

fn ended(directory: &Path) -> PathBuf {
    let id_file = started(directory, KEPT_ONCE_ENDED, "true");
    docker(&["wait", &id_in(&id_file)]);
    id_file
}

fn remove(id_file: &Path) {
    docker(&["rm", "--force", &id_in(id_file)]);
}

fn removed(directory: &Path) -> PathBuf {
    let id_file = ended(directory);
    remove(&id_file);
    id_file
}

#[test]
fn what_a_sandbox_prints_is_returned() {
    assert_eq!(printed_in_a_sandbox_of(IMAGE, "echo printed"), "printed\n");
}

#[test]
fn the_command_is_run_by_a_shell_in_the_sandbox() {
    assert_eq!(
        printed_in_a_sandbox_of(IMAGE, "echo one && echo two"),
        "one\ntwo\n"
    );
}

#[test]
fn what_a_sandbox_writes_to_standard_error_is_not_returned() {
    assert_eq!(printed_in_a_sandbox_of(IMAGE, "echo unseen >&2"), "");
}

#[test]
fn a_sandbox_that_runs_is_running() {
    let directory = temporary_directory();
    let id_file = started(directory.path(), REMOVED_ONCE_ENDED, "sleep 600");
    let running = sandbox_is_running(&id_file);
    remove(&id_file);
    assert!(running);
}

#[test]
fn a_sandbox_that_has_ended_is_not_running() {
    let directory = temporary_directory();
    let id_file = ended(directory.path());
    let running = sandbox_is_running(&id_file);
    remove(&id_file);
    assert!(!running);
}

#[test]
fn a_sandbox_that_was_removed_is_not_running() {
    let directory = temporary_directory();
    assert!(!sandbox_is_running(&removed(directory.path())));
}

#[test]
fn a_sandbox_that_was_removed_is_gone() {
    let directory = temporary_directory();
    assert!(sandbox_is_gone(&removed(directory.path())));
}

#[test]
fn a_sandbox_that_is_removed_while_it_is_waited_for_is_gone() {
    let directory = temporary_directory();
    assert!(sandbox_is_gone(&started(
        directory.path(),
        REMOVED_ONCE_ENDED,
        "sleep 2"
    )));
}

#[test]
fn a_sandbox_that_runs_is_not_gone() {
    let directory = temporary_directory();
    let id_file = started(directory.path(), REMOVED_ONCE_ENDED, "sleep 600");
    let gone = sandbox_is_gone(&id_file);
    remove(&id_file);
    assert!(!gone);
}

#[test]
fn a_sandbox_that_has_ended_and_was_not_removed_is_not_gone() {
    let directory = temporary_directory();
    let id_file = ended(directory.path());
    let gone = sandbox_is_gone(&id_file);
    remove(&id_file);
    assert!(!gone);
}
