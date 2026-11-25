use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn options(directory: &Path) -> Vec<String> {
    [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-verifier-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .iter()
    .map(|(option, value)| format!("{option}={}", value.display()))
    .collect()
}

fn refused(options: impl FnOnce(&Path) -> Vec<String>) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let refused = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, refused)
}

fn with_a_task_that_lacks_its_verifier(directory: &Path) -> Vec<String> {
    let options = options(directory);
    fs::remove_dir_all(directory.join("benchmark/t1/tests")).unwrap();
    options
}

fn with_quiet(directory: &Path) -> Vec<String> {
    options(directory)
        .into_iter()
        .chain(["--quiet".to_owned()])
        .collect()
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_verifier_exits_with_2() {
    let (_directory, refused) = refused(with_a_task_that_lacks_its_verifier);
    assert_eq!(refused.status.code(), Some(2));
}

#[test]
fn quiet_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let (directory, _refused) = refused(with_quiet);
    assert!(!directory.path().join("outputs").exists());
}
