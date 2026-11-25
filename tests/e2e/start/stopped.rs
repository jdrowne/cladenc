use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::shared::STOPPED;

const SAY_THE_CONTAINER_AND_STAY: &str = "hostname; sleep 300";

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--max-verifier-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start
        .arg("start")
        .args(options)
        .stderr(fs::File::create(directory.join("printed")).unwrap());
    start
}

fn signalled(signal: Signal) -> ExitStatus {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start(directory.path(), SAY_THE_CONTAINER_AND_STAY)
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("outputs/running/trial-0-benchmark+t1/stdout");
    test_inputs::wait_for_a_line(&mut cladenc, &id_file, &directory.path().join("printed"));
    kill(Pid::from_raw(cladenc.id().cast_signed()), signal).unwrap();
    cladenc.wait().unwrap()
}

#[test]
fn an_interrupt_during_a_run_gives_exit_status_1() {
    assert_eq!(signalled(Signal::SIGINT).code(), Some(1));
}

#[test]
fn a_request_to_end_during_a_run_gives_exit_status_1() {
    assert_eq!(signalled(Signal::SIGTERM).code(), Some(1));
}

#[test]
fn a_hangup_during_a_run_gives_exit_status_1() {
    assert_eq!(signalled(Signal::SIGHUP).code(), Some(1));
}

#[test]
fn a_quit_during_a_run_gives_exit_status_1() {
    assert_eq!(signalled(Signal::SIGQUIT).code(), Some(1));
}

#[test]
fn a_container_that_docker_does_not_start_gives_exit_status_1() {
    assert_eq!(STOPPED.1.status.code(), Some(1));
}
