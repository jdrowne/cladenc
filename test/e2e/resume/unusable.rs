use std::path::Path;
use std::process::{Command, Output};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

fn start(directory: &Path) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "hostname > id; sleep 300".into()),
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

fn resume(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--run-dir={}", directory.join("run").display()))
        .output()
        .unwrap()
}

fn resumed_while_in_use() -> Output {
    let directory = test_inputs::temporary_directory();
    let mut first = start(directory.path()).spawn().unwrap();
    let id_file = directory.path().join("run/running/evaluation-0-t1/work/id");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    let second = resume(directory.path());
    kill(Pid::from_raw(first.id().cast_signed()), Signal::SIGINT).unwrap();
    first.wait().unwrap();
    second
}

#[test]
fn resume_on_a_missing_run_directory_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(resume(directory.path()).status.code(), Some(2));
}

#[test]
fn a_second_cladenc_on_a_run_directory_in_use_exits_with_2() {
    assert_eq!(resumed_while_in_use().status.code(), Some(2));
}

#[test]
fn a_second_cladenc_on_a_run_directory_in_use_says_that_another_is_using_it() {
    assert_eq!(
        String::from_utf8(resumed_while_in_use().stderr).unwrap(),
        "cladenc: the run directory cannot be used: another cladenc is using it\n"
    );
}
