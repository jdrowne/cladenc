use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tempfile::TempDir;

const SAY_THE_SANDBOX_AND_STAY_IF_HELD: &str = "[ -e hold ] || exit 0; hostname; sleep 300";

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--dataset",
            test_inputs::dataset(directory, &["t1", "t2", "t3"]),
        ),
        ("--diversification-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-diversifications", "1".into()),
        ("--max-concurrent-sandboxes", "1".into()),
        ("--diversification-timeout", "60".into()),
        ("--max-verifier-health-check-retries", "0".into()),
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

static RESUMED_AFTER_AN_INTERRUPT: LazyLock<(TempDir, Output)> = LazyLock::new(|| {
    let directory = stopped_by_an_interrupt();
    let resumed = resume(directory.path());
    (directory, resumed)
});

fn stopped_by_an_interrupt() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), SAY_THE_SANDBOX_AND_STAY_IF_HELD);
    fs::write(
        directory
            .path()
            .join("dataset/t1/environment/workspace/hold"),
        "",
    )
    .unwrap();
    let mut cladenc = start.spawn().unwrap();
    let outputs = directory.path().join("outputs");
    let id_file = outputs.join("running/trial-0-dataset+t1/stdout");
    test_inputs::wait_for_a_line(&mut cladenc, &id_file, &directory.path().join("printed"));
    kill(Pid::from_raw(cladenc.id().cast_signed()), Signal::SIGINT).unwrap();
    cladenc.wait().unwrap();
    fs::remove_file(outputs.join("datasets/dataset/t1/environment/workspace/hold")).unwrap();
    directory
}

#[test]
fn after_an_interrupt_stops_a_run_resume_finishes_it_and_exits_with_0() {
    let (_directory, resumed) = &*RESUMED_AFTER_AN_INTERRUPT;
    assert_eq!(resumed.status.code(), Some(0));
}

#[test]
fn after_an_interrupt_stops_a_run_resume_prints_the_directory_of_a_harness() {
    let (_directory, resumed) = &*RESUMED_AFTER_AN_INTERRUPT;
    assert!(Path::new(str::from_utf8(&resumed.stdout).unwrap().trim_end()).is_dir());
}
