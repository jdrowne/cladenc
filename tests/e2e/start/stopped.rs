use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::LazyLock;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

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
        ("--max-grader-health-check-retries", "0".into()),
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
    test_inputs::wait_for_a_line(&mut cladenc, &id_file, &directory.path().join("printed"));
    kill(Pid::from_raw(cladenc.id().cast_signed()), signal).unwrap();
    let status = cladenc.wait().unwrap();
    (directory, status)
}

static INTERRUPTED: LazyLock<(TempDir, ExitStatus)> = LazyLock::new(|| signalled(Signal::SIGINT));

static HUNG_UP: LazyLock<(TempDir, ExitStatus)> = LazyLock::new(|| signalled(Signal::SIGHUP));

static QUIT: LazyLock<(TempDir, ExitStatus)> = LazyLock::new(|| signalled(Signal::SIGQUIT));

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
    let journal = fs::read_to_string(outputs.join("journal.jsonl")).unwrap_or_default();
    let repository = journal
        .split("\"environment_image_repository\":\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default();
    Command::new("docker")
        .args(["image", "rm", "--force"])
        .args((0..3).map(|place| format!("{repository}:{place}")))
        .output()
        .unwrap();
    (directory, status)
}

#[test]
fn an_interrupt_during_a_run_gives_exit_status_1() {
    let (_directory, status) = &*INTERRUPTED;
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_request_to_end_during_a_run_gives_exit_status_1() {
    let (_directory, status) = signalled(Signal::SIGTERM);
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_hangup_during_a_run_gives_exit_status_1() {
    let (_directory, status) = &*HUNG_UP;
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_quit_during_a_run_gives_exit_status_1() {
    let (_directory, status) = &*QUIT;
    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_quit_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = &*QUIT;
    assert!(test_inputs::container_is_gone(&id_file(directory)));
}

#[test]
fn a_hangup_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = &*HUNG_UP;
    assert!(test_inputs::container_is_gone(&id_file(directory)));
}

#[test]
fn an_interrupt_during_a_run_leaves_no_container_behind() {
    let (directory, _status) = &*INTERRUPTED;
    assert!(test_inputs::container_is_gone(&id_file(directory)));
}

#[test]
fn a_cladenc_that_is_killed_outright_during_a_run_is_ended_by_the_signal() {
    let (_directory, status) = killed_outright();
    assert_eq!(status.signal(), Some(Signal::SIGKILL as i32));
}

#[test]
fn a_container_that_docker_does_not_start_gives_exit_status_1() {
    assert_eq!(STOPPED.1.status.code(), Some(1));
}
