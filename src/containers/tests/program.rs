use std::process::Command;
use std::sync::atomic::AtomicBool;

use crate::containers::ContainerError;
use crate::containers::program::{self, Limits};

#[test]
fn a_program_that_a_signal_ends_is_said_to_be_ended_by_that_signal() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let mut killed = Command::new("sh");
    killed.args(["-c", "kill -s KILL $$"]);
    let ended = program::run(killed, directory.path(), Limits::starting_now(60, &stop));
    assert!(matches!(ended, Err(ContainerError::Signal(9))));
}
