use std::fs;
use std::path::Path;
use std::process::Command;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

fn start(directory: &Path, evaluator: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
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

fn report(directory: &Path) -> Option<i32> {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!("--run-dir={}", directory.join("run").display()))
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn report_exits_with_0_on_a_run_that_stopped_before_its_end() {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), "exit-2.sh").status().unwrap();
    assert_eq!(report(directory.path()), Some(0));
}

#[test]
fn report_exits_with_0_on_a_run_that_is_still_going() {
    let directory = test_inputs::temporary_directory();
    fs::write(directory.path().join("hold"), "").unwrap();
    let mut going = start(directory.path(), "spawn-while-held.sh")
        .spawn()
        .unwrap();
    let pid_file = directory
        .path()
        .join("run/running/evaluation-0-t1/output/pid");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&pid_file));
    let status = report(directory.path());
    kill(Pid::from_raw(going.id().cast_signed()), Signal::SIGINT).unwrap();
    going.wait().unwrap();
    assert_eq!(status, Some(0));
}
