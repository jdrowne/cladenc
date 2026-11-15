use std::fs;
use std::path::Path;
use std::process::Command;

fn options(directory: &Path, run_directory: &Path) -> Vec<String> {
    [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--evaluator", test_inputs::sample("e2e", "exit-0.sh")),
        ("--run-dir", run_directory.to_owned()),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .iter()
    .map(|(option, value)| format!("{option}={}", value.display()))
    .collect()
}

fn status(options: &[String]) -> Option<i32> {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .arg("--quiet")
        .output()
        .unwrap()
        .status
        .code()
}

fn said(options: &[String]) -> String {
    let refused = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    String::from_utf8(refused.stderr).unwrap()
}

fn with_a_task_that_lacks_its_check(directory: &Path) -> Vec<String> {
    let options = options(directory, &directory.join("run"));
    fs::remove_file(directory.join("benchmark/t1/check")).unwrap();
    options
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_check_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_check_is_refused_by_the_id_of_the_task() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    assert!(said(&options).ends_with(": task t1 holds no executable file named check\n"));
}

#[test]
fn a_benchmark_that_is_refused_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

#[test]
fn a_missing_option_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path(), &directory.path().join("run"));
    assert_eq!(status(&options[..options.len() - 1]), Some(2));
}

#[test]
fn a_run_directory_at_which_something_exists_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path(), directory.path());
    assert_eq!(status(&options), Some(2));
}
