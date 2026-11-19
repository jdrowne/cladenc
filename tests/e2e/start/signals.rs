use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

const SAY_THE_CONTAINER_AND_STAY_WHILE_HELD: &str =
    "[ -e hold ] || exit 0; hostname > id; while [ -e hold ]; do sleep 1; done";

fn start(directory: &Path, run_command: &str) -> Vec<String> {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--attempt-timeout", "60".into()),
        ("--holdout-percentage", "50".into()),
        ("--holdout-corroboration-percentage", "50".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    [env!("CARGO_BIN_EXE_cladenc").to_owned(), "start".to_owned()]
        .into_iter()
        .chain(options)
        .collect()
}

#[test]
fn a_run_started_through_nohup_goes_on_to_its_end_after_a_hangup() {
    let directory = test_inputs::temporary_directory();
    let arguments = start(directory.path(), SAY_THE_CONTAINER_AND_STAY_WHILE_HELD);
    fs::write(directory.path().join("benchmark/t1/workspace/hold"), "").unwrap();
    let mut cladenc = Command::new("nohup")
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let outputs = directory.path().join("outputs");
    let work = outputs.join("running/attempt-0-benchmark+t1/work");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&work.join("id")));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGHUP).unwrap();
    fs::remove_file(outputs.join("benchmarks/benchmark/t1/workspace/hold")).unwrap();
    fs::remove_file(work.join("hold")).unwrap();
    assert_eq!(cladenc.wait().unwrap().code(), Some(0));
}

fn status_of_a_run_whose_docker_ends_itself_with(signal: &str) -> Option<i32> {
    let directory = test_inputs::temporary_directory();
    let arguments = start(directory.path(), "exit 0");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "killed").display(),
        env::var("PATH").unwrap()
    );
    Command::new(&arguments[0])
        .args(&arguments[1..])
        .env("PATH", path)
        .env("CLADENC_TEST_SIGNAL_OF_DOCKER", signal)
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn a_docker_that_sends_itself_a_hangup_is_ended_by_it() {
    assert_eq!(
        status_of_a_run_whose_docker_ends_itself_with("HUP"),
        Some(1)
    );
}

#[test]
fn a_docker_that_sends_itself_a_quit_is_ended_by_it() {
    assert_eq!(
        status_of_a_run_whose_docker_ends_itself_with("QUIT"),
        Some(1)
    );
}
