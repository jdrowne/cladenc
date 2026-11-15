use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

fn run(directory: &Path, evaluator: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--tasks", test_inputs::task_file(directory, &["t1"])),
        ("--expander", test_inputs::sample("e2e", "exit-0.sh")),
        ("--evaluator", test_inputs::sample("e2e", evaluator)),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-expansions", "0".into()),
        ("--workers", "1".into()),
        ("--expansion-timeout", "60".into()),
        ("--evaluation-timeout", "60".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut run = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    run.arg("run").args(options).arg("--quiet");
    run
}

fn resume(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--run-dir={}", directory.join("run").display()))
        .arg("--quiet")
        .output()
        .unwrap()
}

fn interrupted() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let hold = directory.path().join("hold");
    fs::write(&hold, "").unwrap();
    let mut cladenc = run(directory.path(), "spawn-while-held.sh")
        .spawn()
        .unwrap();
    let pid_file = directory
        .path()
        .join("run/running/evaluation-0-t1/output/pid");
    test_inputs::within_ten_seconds(|| test_inputs::pid_is_written(&pid_file));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGINT).unwrap();
    cladenc.wait().unwrap();
    fs::remove_file(hold).unwrap();
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
fn after_an_interrupt_stops_a_run_resume_ends_the_journal_with_search_finished() {
    let directory = interrupted();
    resume(directory.path());
    let journal = fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap();
    assert!(
        journal
            .lines()
            .last()
            .unwrap()
            .starts_with(r#"{"record":"search_finished""#)
    );
}

#[test]
fn a_search_that_stops_again_under_resume_exits_with_1() {
    let directory = test_inputs::temporary_directory();
    run(directory.path(), "exit-2.sh").status().unwrap();
    assert_eq!(resume(directory.path()).status.code(), Some(1));
}
