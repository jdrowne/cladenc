use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

fn run(directory: &Path, evaluator: &str) -> Vec<String> {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--tasks", test_inputs::task_file(directory, &["t1"])),
        ("--mutator", test_inputs::sample("e2e", "exit-0.sh")),
        ("--evaluator", test_inputs::sample("e2e", evaluator)),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-mutations", "0".into()),
        ("--workers", "1".into()),
        ("--mutation-timeout", "60".into()),
        ("--evaluation-timeout", "60".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    [
        env!("CARGO_BIN_EXE_cladenc").to_owned(),
        "run".to_owned(),
        "--quiet".to_owned(),
    ]
    .into_iter()
    .chain(options)
    .collect()
}

#[test]
fn a_run_started_through_nohup_goes_on_to_its_end_after_a_hangup() {
    let directory = test_inputs::temporary_directory();
    let hold = directory.path().join("hold");
    fs::write(&hold, "").unwrap();
    let mut cladenc = Command::new("nohup")
        .args(run(directory.path(), "wait-while-held.sh"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid_file = directory
        .path()
        .join("run/running/evaluation-0-t1/output/pid");
    test_inputs::within_ten_seconds(|| test_inputs::pid_is_written(&pid_file));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGHUP).unwrap();
    fs::remove_file(hold).unwrap();
    assert_eq!(cladenc.wait().unwrap().code(), Some(0));
}

#[test]
fn a_program_that_sends_itself_a_hangup_is_ended_by_it() {
    let directory = test_inputs::temporary_directory();
    let arguments = run(directory.path(), "hang-up-self.sh");
    let ended = Command::new(&arguments[0])
        .args(&arguments[1..])
        .output()
        .unwrap();
    assert_eq!(ended.status.code(), Some(1));
}
