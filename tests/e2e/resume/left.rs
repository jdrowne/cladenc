use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

const SAY_THE_CONTAINER_AND_STAY_IF_HELD: &str = "[ -e hold ] || exit 0; hostname; sleep 300";

struct Left {
    directory: TempDir,
    id_file: PathBuf,
}

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
        ("--procreation-timeout", "600".into()),
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

fn resume(outputs_directory: &Path) -> Command {
    let mut resume = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    resume
        .arg("resume")
        .arg("--outputs-dir")
        .arg(outputs_directory);
    resume
}

fn killed_outright() -> Left {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), SAY_THE_CONTAINER_AND_STAY_IF_HELD);
    fs::write(
        directory
            .path()
            .join("benchmark/t1/environment/workspace/hold"),
        "",
    )
    .unwrap();
    let mut cladenc = start.spawn().unwrap();
    let outputs = directory.path().join("outputs");
    let written = outputs.join("running/attempt-0-benchmark+t1/stdout");
    test_inputs::wait_for_a_line(&mut cladenc, &written, &directory.path().join("printed"));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGKILL).unwrap();
    cladenc.wait().unwrap();
    let id_file = directory.path().join("id-of-the-container-left");
    fs::copy(&written, &id_file).unwrap();
    fs::remove_file(outputs.join("benchmarks/benchmark/t1/environment/workspace/hold")).unwrap();
    Left { directory, id_file }
}

fn resumed_after_being_killed_outright() -> (Left, Output) {
    let left = killed_outright();
    let output = resume(&left.directory.path().join("outputs"))
        .output()
        .unwrap();
    (left, output)
}

fn stopped() -> TempDir {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit 125").status().unwrap();
    directory
}

fn resumed_with_no_docker() -> Output {
    let directory = stopped();
    let empty = directory.path().join("no-programs");
    fs::create_dir(&empty).unwrap();
    resume(&directory.path().join("outputs"))
        .env("PATH", empty)
        .output()
        .unwrap()
}

#[test]
fn resume_removes_the_container_that_a_cladenc_killed_outright_left_running() {
    let (left, _output) = resumed_after_being_killed_outright();
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn after_cladenc_was_killed_outright_resume_finishes_the_directed_evolution_and_exits_with_0() {
    let (_left, output) = resumed_after_being_killed_outright();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn resume_says_how_many_containers_it_removed() {
    let (_left, output) = resumed_after_being_killed_outright();
    assert_eq!(
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .next()
            .unwrap_or_default(),
        "cladenc: removed 1 container that the directed evolution left running"
    );
}

#[test]
fn a_directed_evolution_reached_through_a_symbolic_link_has_its_containers_removed_too() {
    let left = killed_outright();
    let link = left.directory.path().join("link-to-the-outputs-directory");
    symlink(left.directory.path().join("outputs"), &link).unwrap();
    resume(&link).output().unwrap();
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn resume_says_nothing_of_containers_where_none_was_left() {
    let directory = stopped();
    let output = resume(&directory.path().join("outputs")).output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).contains("cladenc: removed"));
}

#[test]
fn a_resume_with_no_docker_to_call_exits_with_2() {
    assert_eq!(resumed_with_no_docker().status.code(), Some(2));
}
