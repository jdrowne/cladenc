use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn ran() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let options = [
        ("--progenitor", test_inputs::progenitor(directory.path())),
        ("--tasks", test_inputs::task_file(directory.path(), &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            "echo \"$CLADENC_HARNESS\" > where; pwd >> where".into(),
        ),
        (
            "--evaluator",
            test_inputs::sample("e2e", "record-arguments.sh"),
        ),
        ("--run-dir", directory.path().join("run")),
        ("--budget", "1".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "1".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .arg("--quiet")
        .output()
        .unwrap();
    directory
}

#[test]
fn a_harness_is_started_in_a_container_that_holds_its_source_code_and_the_copy_to_improve() {
    let directory = ran();
    assert_eq!(
        fs::read_to_string(directory.path().join("run/harnesses/1/where")).unwrap(),
        "/harness\n/work\n"
    );
}

#[test]
fn the_evaluator_is_given_the_harness_the_task_and_the_output() {
    let directory = ran();
    let run = directory.path().join("run");
    let evaluated = fs::read_dir(run.join("evaluations"))
        .unwrap()
        .map(|harness| harness.unwrap().path().join("t1/output/arguments"))
        .find(|arguments| arguments.is_file())
        .unwrap();
    let harness = evaluated
        .strip_prefix(run.join("evaluations"))
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(
        fs::read_to_string(&evaluated).unwrap(),
        format!(
            "{0}/running/evaluation-{harness}-t1/harness\n\
             t1\n\
             {0}/running/evaluation-{harness}-t1/output\n",
            run.display()
        )
    );
}
