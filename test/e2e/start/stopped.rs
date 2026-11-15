use std::path::Path;
use std::process::{Command, ExitStatus};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

fn start(directory: &Path, evaluator: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--tasks", test_inputs::task_file(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--evaluator", test_inputs::sample("e2e", evaluator)),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--evaluation-timeout", "60".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).arg("--quiet");
    start
}

fn signalled(signal: Signal) -> (TempDir, ExitStatus) {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start(directory.path(), "spawn.sh").spawn().unwrap();
    let pid_file = directory
        .path()
        .join("run/running/evaluation-0-t1/output/pid");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&pid_file));
    kill(Pid::from_raw(cladenc.id().cast_signed()), signal).unwrap();
    let status = cladenc.wait().unwrap();
    (directory, status)
}

#[test]
fn an_interrupt_during_a_run_gives_exit_status_1() {
    let (_directory, status) = signalled(Signal::SIGINT);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_request_to_end_during_a_run_gives_exit_status_1() {
    let (_directory, status) = signalled(Signal::SIGTERM);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_hangup_during_a_run_gives_exit_status_1() {
    let (_directory, status) = signalled(Signal::SIGHUP);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_quit_during_a_run_gives_exit_status_1() {
    let (_directory, status) = signalled(Signal::SIGQUIT);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_quit_during_a_run_leaves_no_sample_program_running() {
    let (directory, _status) = signalled(Signal::SIGQUIT);
    assert!(test_inputs::process_is_gone(
        &directory
            .path()
            .join("run/running/evaluation-0-t1/output/pid")
    ));
}

#[test]
fn a_hangup_during_a_run_leaves_no_sample_program_running() {
    let (directory, _status) = signalled(Signal::SIGHUP);
    assert!(test_inputs::process_is_gone(
        &directory
            .path()
            .join("run/running/evaluation-0-t1/output/pid")
    ));
}

#[test]
fn an_interrupt_during_a_run_leaves_no_sample_program_running() {
    let (directory, _status) = signalled(Signal::SIGINT);
    assert!(test_inputs::process_is_gone(
        &directory
            .path()
            .join("run/running/evaluation-0-t1/output/pid")
    ));
}

#[test]
fn an_evaluator_that_exits_with_a_status_other_than_0_or_1_gives_exit_status_1() {
    let directory = test_inputs::temporary_directory();
    let status = start(directory.path(), "exit-2.sh").status().unwrap();
    assert_eq!(status.code(), Some(1));
}
