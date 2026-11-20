use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn run_command_the_help_gives(command: &str) -> String {
    let help = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .args(["start", "--help"])
        .output()
        .unwrap();
    String::from_utf8(help.stdout)
        .unwrap()
        .split_once("--harness-run-command '")
        .and_then(|(_, after)| after.split_once('\''))
        .map(|(given, _)| given.replace("<COMMAND>", command))
        .unwrap()
}

fn ran() -> TempDir {
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
        ("--docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            run_command_the_help_gives("echo attempted > attempted").into(),
        ),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "2".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--attempt-timeout", "30".into()),
        ("--holdout-percentage", "50".into()),
        ("--holdout-corroboration-percentage", "50".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    directory
}

fn script_of(directory: &TempDir, harness: &str) -> String {
    fs::read_to_string(
        directory
            .path()
            .join("outputs/harnesses")
            .join(harness)
            .join("harness.sh"),
    )
    .unwrap()
}

fn names_in_harness(directory: &TempDir, harness: &str) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(directory.path().join("outputs/harnesses").join(harness))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name != "procreated")
            .collect();
    names.sort();
    names
}

#[test]
fn a_run_with_the_run_command_the_help_gives_creates_as_many_children_as_the_most_procreations() {
    let directory = ran();
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
    assert_eq!(
        journal
            .lines()
            .filter(|line| line.contains(r#""record":"procreation_finished""#))
            .filter(|line| line.contains(r#""outcome":"created""#))
            .count(),
        2
    );
}

#[test]
fn a_run_with_the_run_command_the_help_gives_adds_to_one_file_of_each_child_and_names_no_other() {
    let directory = ran();
    assert_eq!(
        ["1", "2"].map(|child| names_in_harness(&directory, child)),
        [
            names_in_harness(&directory, "0"),
            names_in_harness(&directory, "0")
        ]
    );
}

#[test]
fn a_run_with_the_run_command_the_help_gives_leaves_the_script_of_each_child_unchanged() {
    let directory = ran();
    assert_eq!(
        ["1", "2"].map(|child| script_of(&directory, child)),
        [script_of(&directory, "0"), script_of(&directory, "0")]
    );
}

#[test]
fn a_run_with_the_run_command_the_help_gives_adds_one_line_to_a_file_of_a_child() {
    let directory = ran();
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/1/procreated")).unwrap(),
        "\n"
    );
}
