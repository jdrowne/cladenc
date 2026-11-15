use std::path::Path;
use std::process::Command;

fn options(directory: &Path, run_directory: &Path) -> Vec<String> {
    [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--tasks", test_inputs::task_file(directory, &["t1"])),
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
