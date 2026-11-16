use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

const SAY_THE_CONTAINER_AND_STAY_IF_HELD: &str = "[ -e hold ] || exit 0; hostname > id; sleep 300";

struct Left {
    directory: TempDir,
    id_file: PathBuf,
}

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "600".into()),
        ("--evaluation-timeout", "600".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).stderr(Stdio::null());
    start
}

fn resume(run_directory: &Path) -> Command {
    let mut resume = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    resume.arg("resume").arg("--run-dir").arg(run_directory);
    resume
}

fn killed_outright() -> Left {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), SAY_THE_CONTAINER_AND_STAY_IF_HELD);
    fs::write(directory.path().join("benchmark/t1/files/hold"), "").unwrap();
    let mut cladenc = start.spawn().unwrap();
    let run = directory.path().join("run");
    let written = run.join("running/evaluation-0-benchmark+t1/work/id");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&written));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGKILL).unwrap();
    cladenc.wait().unwrap();
    let id_file = directory.path().join("id-of-the-container-left");
    fs::copy(&written, &id_file).unwrap();
    fs::remove_file(run.join("benchmarks/benchmark/t1/files/hold")).unwrap();
    Left { directory, id_file }
}

fn resumed_after_being_killed_outright() -> (Left, Output) {
    let left = killed_outright();
    let output = resume(&left.directory.path().join("run")).output().unwrap();
    (left, output)
}

fn stopped() -> TempDir {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit 125").status().unwrap();
    directory
}

fn resumed_with_the_docker_of(sample: &str, directory: &TempDir) -> Output {
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", sample).display(),
        env::var("PATH").unwrap()
    );
    resume(&directory.path().join("run"))
        .env("PATH", path)
        .env(
            "CLADENC_TEST_ARGUMENTS_OF_DOCKER",
            directory.path().join("arguments-of-docker"),
        )
        .output()
        .unwrap()
}

fn resumed_with_a_docker_that(sample: &str) -> Output {
    resumed_with_the_docker_of(sample, &stopped())
}

fn resumed_with_no_docker() -> Output {
    let directory = stopped();
    let empty = directory.path().join("no-programs");
    fs::create_dir(&empty).unwrap();
    resume(&directory.path().join("run"))
        .env("PATH", empty)
        .output()
        .unwrap()
}

fn last_line_said(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .last()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn resume_removes_the_container_that_a_cladenc_killed_outright_left_running() {
    let (left, _output) = resumed_after_being_killed_outright();
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn after_cladenc_was_killed_outright_resume_finishes_the_search_and_exits_with_0() {
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
        "cladenc: removed 1 container that the search left running"
    );
}

#[test]
fn a_search_reached_through_a_symbolic_link_has_its_containers_removed_too() {
    let left = killed_outright();
    let link = left.directory.path().join("link-to-the-run-directory");
    symlink(left.directory.path().join("run"), &link).unwrap();
    resume(&link).output().unwrap();
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn resume_says_nothing_of_containers_where_none_was_left() {
    let directory = stopped();
    let output = resume(&directory.path().join("run")).output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).contains("cladenc: removed"));
}

#[test]
fn resume_asks_docker_first_for_the_containers_that_carry_the_label_of_its_search() {
    let directory = stopped();
    let run = fs::canonicalize(directory.path().join("run")).unwrap();
    resumed_with_the_docker_of("fake", &directory);
    let arguments = fs::read_to_string(directory.path().join("arguments-of-docker")).unwrap();
    assert!(arguments.starts_with(&format!(
        "ps\n--all\n--quiet\n--filter\nlabel=cladenc.search={}\n",
        run.display()
    )));
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_exits_with_2() {
    assert_eq!(
        resumed_with_a_docker_that("unreachable").status.code(),
        Some(2)
    );
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_says_what_docker_said() {
    assert_eq!(
        last_line_said(&resumed_with_a_docker_that("unreachable")),
        "cladenc: the containers that the search left running cannot be removed: Cannot connect \
         to the Docker daemon"
    );
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_continues_nothing() {
    let directory = stopped();
    let journal = directory.path().join("run/journal.jsonl");
    let before = fs::read_to_string(&journal).unwrap();
    resumed_with_the_docker_of("unreachable", &directory);
    assert_eq!(fs::read_to_string(&journal).unwrap(), before);
}

#[test]
fn a_resume_whose_docker_does_not_remove_a_container_exits_with_2() {
    assert_eq!(
        resumed_with_a_docker_that("unremoved").status.code(),
        Some(2)
    );
}

#[test]
fn a_resume_whose_docker_does_not_remove_a_container_says_what_docker_said() {
    assert_eq!(
        last_line_said(&resumed_with_a_docker_that("unremoved")),
        "cladenc: the containers that the search left running cannot be removed: cannot remove \
         container 0123456789ab"
    );
}

#[test]
fn a_resume_with_no_docker_to_call_exits_with_2() {
    assert_eq!(resumed_with_no_docker().status.code(), Some(2));
}
