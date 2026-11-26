use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

pub static ENDED: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| ran(test_inputs::SOLVE_EVERY_TASK));

pub static STOPPED: LazyLock<(TempDir, Output)> = LazyLock::new(|| ran("exit 125"));

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
        ("--diversification-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-diversifications", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--diversification-timeout", "30".into()),
        ("--max-verifier-health-check-retries", "0".into()),
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
