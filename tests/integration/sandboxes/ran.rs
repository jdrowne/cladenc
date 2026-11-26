use std::fs;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::sandboxes::{Docker, Ending, Limits, Program, SandboxError};
use tempfile::TempDir;

struct Ran {
    directory: TempDir,
    ended: Result<Ending, SandboxError>,
}

fn sh(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]);
    command
}

fn ran_until(script: &str, input: &str, time_limit: u64, stop: bool) -> Ran {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(stop);
    let ended = Program.run(
        sh(script),
        input,
        directory.path(),
        Limits::starting_now(time_limit, &stop),
    );
    Ran { directory, ended }
}

fn ran(script: &str, input: &str) -> Ran {
    ran_until(script, input, 60, false)
}

static PRINTED_AND_SAID: LazyLock<Ran> = LazyLock::new(|| ran("echo printed; echo said >&2", ""));

fn kept(ran: &Ran, name: &str) -> String {
    fs::read_to_string(ran.directory.path().join(name)).unwrap()
}

#[test]
fn a_program_that_exits_is_said_to_have_exited_with_its_status() {
    assert!(matches!(ran("exit 3", "").ended, Ok(Ending::Exited(3))));
}

#[test]
fn a_program_that_a_signal_ends_is_said_to_be_ended_by_that_signal() {
    assert!(matches!(
        ran("kill -s KILL $$", "").ended,
        Err(SandboxError::Signal(9))
    ));
}

#[test]
fn a_program_that_sends_itself_a_hangup_is_ended_by_it() {
    assert!(matches!(
        ran("kill -s HUP $$", "").ended,
        Err(SandboxError::Signal(1))
    ));
}

#[test]
fn a_program_that_sends_itself_a_quit_is_ended_by_it() {
    assert!(matches!(
        ran("kill -s QUIT $$", "").ended,
        Err(SandboxError::Signal(3))
    ));
}

#[test]
fn a_program_still_running_once_a_stop_is_asked_for_is_ended_and_said_to_be_stopped() {
    assert!(matches!(
        ran_until("sleep 300", "", 60, true).ended,
        Err(SandboxError::Stopped)
    ));
}

#[test]
fn a_program_still_running_past_its_time_limit_is_ended_and_said_to_have_timed_out() {
    assert!(matches!(
        ran_until("sleep 300", "", 0, false).ended,
        Ok(Ending::TimedOut)
    ));
}

#[test]
fn what_a_program_prints_is_kept_in_the_file_named_stdout() {
    assert_eq!(kept(&PRINTED_AND_SAID, "stdout"), "printed\n");
}

#[test]
fn what_a_program_says_went_wrong_is_kept_in_the_file_named_stderr() {
    assert_eq!(kept(&PRINTED_AND_SAID, "stderr"), "said\n");
}

#[test]
fn a_program_reads_what_it_is_given_on_its_standard_input() {
    let ran = ran("cat", "MODE=strict\n");
    assert_eq!(kept(&ran, "stdout"), "MODE=strict\n");
}

#[test]
fn a_program_with_no_directory_to_keep_what_it_prints_is_not_started() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let ended = Program.run(
        sh("exit 0"),
        "",
        &directory.path().join("missing"),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(ended, Err(SandboxError::Io(_))));
}

#[test]
fn a_program_that_cannot_be_started_is_an_error() {
    let directory = test_inputs::temporary_directory();
    let stop = AtomicBool::new(false);
    let ended = Program.run(
        Command::new("/no/such/program"),
        "",
        directory.path(),
        Limits::starting_now(60, &stop),
    );
    assert!(matches!(ended, Err(SandboxError::Io(_))));
}
