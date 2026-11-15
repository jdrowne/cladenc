use std::path::Path;
use std::process::Command;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

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
    start(directory.path(), "exit 125").status().unwrap();
    assert_eq!(report(directory.path()), Some(0));
}

#[test]
fn report_exits_with_0_on_a_run_that_is_still_going() {
    let directory = test_inputs::temporary_directory();
    let mut going = start(directory.path(), "hostname > id; sleep 300")
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("run/running/evaluation-0-benchmark+t1/work/id");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    let status = report(directory.path());
    kill(Pid::from_raw(going.id().cast_signed()), Signal::SIGINT).unwrap();
    going.wait().unwrap();
    assert_eq!(status, Some(0));
}
