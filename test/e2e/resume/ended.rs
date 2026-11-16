use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn ended() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let options = [
        ("--progenitor", test_inputs::progenitor(directory.path())),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--run-dir", directory.path().join("run")),
        ("--budget", "10".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, output)
}

fn resume(directory: &Path) -> Output {
    resume_with(directory, None)
}

fn resume_with(directory: &Path, another: Option<&str>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--run-dir={}", directory.join("run").display()))
        .args(another)
        .output()
        .unwrap()
}

#[test]
fn resume_on_a_run_that_ended_prints_the_path_the_run_printed() {
    let (directory, run) = ended();
    assert_eq!(resume(directory.path()).stdout, run.stdout);
}

#[test]
fn resume_on_a_run_that_ended_exits_with_0() {
    let (directory, _run) = ended();
    assert_eq!(resume(directory.path()).status.code(), Some(0));
}

#[test]
fn resume_on_a_run_that_ended_leaves_the_journal_unchanged() {
    let (directory, _run) = ended();
    let journal = directory.path().join("run/journal.jsonl");
    let before = fs::read(&journal).unwrap();
    resume(directory.path());
    assert_eq!(fs::read(&journal).unwrap(), before);
}

#[test]
fn resume_on_a_run_that_ended_says_on_standard_error_that_the_search_had_finished() {
    let (directory, _run) = ended();
    assert!(
        String::from_utf8(resume(directory.path()).stderr)
            .unwrap()
            .starts_with("cladenc: the search had finished: its best harness is harness ")
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_exits_with_2() {
    let (directory, _run) = ended();
    assert_eq!(
        resume_with(directory.path(), Some("--quiet")).status.code(),
        Some(2)
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_is_refused_as_an_argument_that_is_not_expected() {
    let (directory, _run) = ended();
    assert!(
        String::from_utf8(resume_with(directory.path(), Some("--quiet")).stderr)
            .unwrap()
            .starts_with("error: unexpected argument '--quiet' found\n")
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_prints_nothing_on_standard_output() {
    let (directory, _run) = ended();
    assert_eq!(
        resume_with(directory.path(), Some("--quiet")).stdout,
        Vec::<u8>::new()
    );
}
