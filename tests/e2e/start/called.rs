use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use tempfile::TempDir;

const TOLD: &str = "a-model http://models.example/v1";
const SAY_WHAT_THE_HARNESS_IS_TOLD_OF_THE_MODEL: &str =
    "echo \"$CLADENC_MODEL $CLADENC_MODEL_URL ${CLADENC_MODEL_KEY-none}\" > model";

fn start(directory: &Path) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--dataset",
            test_inputs::dataset(directory, &["t1", "t2", "t3"]),
        ),
        ("--diversification-docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            SAY_WHAT_THE_HARNESS_IS_TOLD_OF_THE_MODEL.into(),
        ),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-diversifications", "1".into()),
        ("--max-concurrent-sandboxes", "1".into()),
        ("--diversification-timeout", "30".into()),
        ("--max-verifier-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

static RAN_WITH_NO_FILE_OF_A_KEY_AND_A_KEY_IN_THE_ENVIRONMENT: LazyLock<TempDir> =
    LazyLock::new(|| {
        let directory = test_inputs::temporary_directory();
        start(directory.path())
            .env("CLADENC_MODEL_KEY", "a-key-from-elsewhere")
            .output()
            .unwrap();
        directory
    });

fn kept_by_the_harness(directory: &TempDir, file: &str) -> String {
    fs::read_to_string(directory.path().join("outputs/harnesses/1").join(file)).unwrap()
}

fn left_by_the_trial(directory: &TempDir, file: &str) -> String {
    let kept = test_inputs::kept_trial(&directory.path().join("outputs"), 0, "dataset/t1");
    fs::read_to_string(kept.join("work").join(file)).unwrap()
}

#[test]
fn a_harness_is_given_no_key_that_no_file_holds_though_the_environment_of_cladenc_holds_one() {
    let directory = &*RAN_WITH_NO_FILE_OF_A_KEY_AND_A_KEY_IN_THE_ENVIRONMENT;
    assert_eq!(
        kept_by_the_harness(directory, "model"),
        format!("{TOLD} none\n")
    );
}

#[test]
fn the_harness_of_a_trial_is_given_no_key_that_no_file_holds_either() {
    let directory = &*RAN_WITH_NO_FILE_OF_A_KEY_AND_A_KEY_IN_THE_ENVIRONMENT;
    assert_eq!(
        left_by_the_trial(directory, "model"),
        format!("{TOLD} none\n")
    );
}
