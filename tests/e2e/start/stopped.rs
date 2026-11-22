use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

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
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).stderr(Stdio::null());
    start
}

fn id_file(directory: &TempDir) -> PathBuf {
    directory
        .path()
        .join("outputs/running/attempt-0-benchmark+t1/stdout")
}

fn signalled(signal: Signal) -> (TempDir, ExitStatus) {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start(directory.path(), SAY_THE_CONTAINER_AND_STAY)
        .spawn()
        .unwrap();
    let id_file = id_file(&directory);
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    kill(Pid::from_raw(cladenc.id().cast_signed()), signal).unwrap();
    let status = cladenc.wait().unwrap();
    (directory, status)
}

fn killed_outright() -> (TempDir, ExitStatus) {
    let (directory, status) = signalled(Signal::SIGKILL);
    let outputs = fs::canonicalize(directory.path().join("outputs")).unwrap();
    let listed = Command::new("docker")
        .args(["ps", "--all", "--quiet", "--filter"])
        .arg(format!(
            "label=cladenc.directed-evolution={}",
            outputs.display()
        ))
        .output()
        .unwrap();
    Command::new("docker")
        .args(["rm", "--force"])
        .args(String::from_utf8(listed.stdout).unwrap().lines())
        .output()
        .unwrap();
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
fn a_quit_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = signalled(Signal::SIGQUIT);
    assert!(test_inputs::container_is_gone(&id_file(&directory)));
}

#[test]
fn a_hangup_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = signalled(Signal::SIGHUP);
    assert!(test_inputs::container_is_gone(&id_file(&directory)));
}

#[test]
fn an_interrupt_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = signalled(Signal::SIGINT);
    assert!(test_inputs::container_is_gone(&id_file(&directory)));
}

#[test]
fn a_cladenc_that_is_killed_outright_during_a_run_is_ended_by_the_signal() {
    let (_directory, status) = killed_outright();
    assert_eq!(status.signal(), Some(Signal::SIGKILL as i32));
}

#[test]
fn a_container_that_docker_does_not_start_gives_exit_status_1() {
    let directory = test_inputs::temporary_directory();
    let status = start(directory.path(), "exit 125").status().unwrap();
    assert_eq!(status.code(), Some(1));
}
