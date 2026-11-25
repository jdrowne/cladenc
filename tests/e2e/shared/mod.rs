use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

pub static ENDED: LazyLock<(TempDir, Output)> = LazyLock::new(solved_every_task);

pub static UNCHANGED: LazyLock<(TempDir, Output)> = LazyLock::new(|| ran("exit 0"));

pub static STOPPED: LazyLock<(TempDir, Output)> = LazyLock::new(|| ran("exit 125"));

pub fn solved_every_task() -> (TempDir, Output) {
    ran(test_inputs::SOLVE_EVERY_TASK)
}

fn ran(run_command: &str) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let options = [
        (
            "--primogenitor",
            test_inputs::primogenitor(directory.path()),
        ),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, output)
}
