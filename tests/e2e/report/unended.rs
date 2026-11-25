use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::shared::STOPPED;

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
        ("--procreation-timeout", "60".into()),
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

fn report(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap()
}

static REPORT_OF_ONE_THAT_STOPPED_BEFORE_ITS_END: LazyLock<Output> =
    LazyLock::new(|| report(STOPPED.0.path()));

#[test]
fn report_exits_with_2_on_a_directed_evolution_that_stopped_before_its_end() {
    assert_eq!(
        REPORT_OF_ONE_THAT_STOPPED_BEFORE_ITS_END.status.code(),
        Some(2)
    );
}

#[test]
fn report_says_of_a_directed_evolution_that_stopped_before_its_end_to_call_status() {
    assert_eq!(
        str::from_utf8(&REPORT_OF_ONE_THAT_STOPPED_BEFORE_ITS_END.stderr).unwrap(),
        "cladenc: the directed evolution has not ended: call `cladenc status` to read its state \
         so far\n"
    );
}

#[test]
fn report_prints_nothing_on_standard_output_of_a_directed_evolution_that_stopped_before_its_end() {
    assert_eq!(
        REPORT_OF_ONE_THAT_STOPPED_BEFORE_ITS_END.stdout,
        Vec::<u8>::new()
    );
}

#[test]
fn report_exits_with_2_on_a_directed_evolution_that_is_still_going() {
    let directory = test_inputs::temporary_directory();
    let mut going = start(directory.path(), "hostname; sleep 300")
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("outputs/running/attempt-0-benchmark+t1/stdout");
    test_inputs::wait_for_a_line(&mut going, &id_file, &directory.path().join("printed"));
    let status = report(directory.path()).status.code();
    kill(Pid::from_raw(going.id().cast_signed()), Signal::SIGINT).unwrap();
    going.wait().unwrap();
    assert_eq!(status, Some(2));
}
