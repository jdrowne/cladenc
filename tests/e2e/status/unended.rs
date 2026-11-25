use std::path::Path;
use std::process::{Command, Output};

use crate::shared::STOPPED;

fn status(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("status")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap()
}

fn lines_of(directory: &Path) -> Vec<String> {
    String::from_utf8(status(directory).stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn status_exits_with_0_on_a_directed_evolution_that_stopped_before_its_end() {
    let (directory, _run) = &*STOPPED;
    assert_eq!(status(directory.path()).status.code(), Some(0));
}

#[test]
fn status_names_the_harness_that_is_ahead_so_far() {
    let (directory, _run) = &*STOPPED;
    assert_eq!(
        lines_of(directory.path())[2],
        "started 0 of 1 procreation, harness ahead so far 0"
    );
}
