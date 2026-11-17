use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

const SAY_THE_CONTAINER_AND_STAY_IF_HELD: &str = "[ -e hold ] || exit 0; hostname > id; sleep 300";

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--attempt-timeout", "60".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).stderr(Stdio::null());
    start
}

fn resume(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap()
}

fn interrupted() -> TempDir {
    stopped_by(Signal::SIGINT)
}

fn stopped_by(signal: Signal) -> TempDir {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), SAY_THE_CONTAINER_AND_STAY_IF_HELD);
    fs::write(directory.path().join("benchmark/t1/workspace/hold"), "").unwrap();
    let mut cladenc = start.spawn().unwrap();
    let outputs = directory.path().join("outputs");
    let id_file = outputs.join("running/attempt-0-benchmark+t1/work/id");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    kill(Pid::from_raw(cladenc.id().cast_signed()), signal).unwrap();
    cladenc.wait().unwrap();
    fs::remove_file(outputs.join("benchmarks/benchmark/t1/workspace/hold")).unwrap();
    directory
}

#[test]
fn after_an_interrupt_stops_a_run_resume_finishes_it_and_exits_with_0() {
    let directory = interrupted();
    assert_eq!(resume(directory.path()).status.code(), Some(0));
}

#[test]
fn after_an_interrupt_stops_a_run_resume_prints_the_directory_of_a_harness() {
    let directory = interrupted();
    let output = resume(directory.path());
    assert!(Path::new(String::from_utf8(output.stdout).unwrap().trim_end()).is_dir());
}

#[test]
fn after_an_interrupt_stops_a_run_resume_ends_the_journal_with_directed_evolution_finished() {
    let directory = interrupted();
    resume(directory.path());
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
    assert!(
        journal
            .lines()
            .last()
            .unwrap()
            .starts_with(r#"{"record":"directed_evolution_finished""#)
    );
}

#[test]
fn after_a_hangup_stops_a_run_resume_finishes_it_and_exits_with_0() {
    let directory = stopped_by(Signal::SIGHUP);
    assert_eq!(resume(directory.path()).status.code(), Some(0));
}

#[test]
fn after_a_quit_stops_a_run_resume_finishes_it_and_exits_with_0() {
    let directory = stopped_by(Signal::SIGQUIT);
    assert_eq!(resume(directory.path()).status.code(), Some(0));
}

#[test]
fn a_directed_evolution_that_stops_again_under_resume_exits_with_1() {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit 125").status().unwrap();
    assert_eq!(resume(directory.path()).status.code(), Some(1));
}
