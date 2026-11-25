use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

const TOLD: &str = "a-model http://models.example/v1";
const SAY_WHAT_THE_HARNESS_IS_GIVEN: &str = "echo \"$CLADENC_HARNESS\" > where; pwd >> where; \
                                             echo \"$CLADENC_MODEL $CLADENC_MODEL_URL \
                                             ${CLADENC_MODEL_KEY-none}\" > model";
const SAY_WHAT_THE_GRADER_IS_GIVEN: &str = "ls /tests; pwd; echo \"${CLADENC_MODEL-none} \
                                           ${CLADENC_MODEL_URL-none} ${CLADENC_MODEL_KEY-none}\" \
                                           >&2";

fn start(directory: &Path, key_file: Option<PathBuf>) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            SAY_WHAT_THE_HARNESS_IS_GIVEN.into(),
        ),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .into_iter()
    .chain(key_file.map(|file| ("--model-key-file", file)))
    .map(|(option, value)| format!("{option}={}", value.display()));
    test_inputs::grader(directory, "t1", SAY_WHAT_THE_GRADER_IS_GIVEN);
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

static RAN: LazyLock<(TempDir, Output)> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    let output = start(directory.path(), Some(key_file)).output().unwrap();
    (directory, output)
});

static RAN_WITH_NO_FILE_OF_A_KEY_AND_A_KEY_IN_THE_ENVIRONMENT: LazyLock<TempDir> =
    LazyLock::new(|| {
        let directory = test_inputs::temporary_directory();
        start(directory.path(), None)
            .env("CLADENC_MODEL_KEY", "a-key-from-elsewhere")
            .output()
            .unwrap();
        directory
    });

fn kept_by_the_harness(directory: &TempDir, file: &str) -> String {
    fs::read_to_string(directory.path().join("outputs/harnesses/1").join(file)).unwrap()
}

fn kept_with_the_attempt(directory: &TempDir, file: &str) -> String {
    let kept = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t1");
    fs::read_to_string(kept.join(file)).unwrap()
}

fn left_by_the_attempt(directory: &TempDir, file: &str) -> String {
    kept_with_the_attempt(directory, &format!("work/{file}"))
}

#[test]
fn a_harness_is_started_in_a_container_that_holds_its_source_code_and_the_copy_to_improve() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        kept_by_the_harness(directory, "where"),
        "/var/opt/cladenc/harness\n/var/opt/cladenc/work\n"
    );
}

#[test]
fn a_harness_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        kept_by_the_harness(directory, "model"),
        format!("{TOLD} a-made-up-key\n")
    );
}

#[test]
fn the_harness_of_an_attempt_is_started_in_the_working_directory_of_the_environment_of_its_task() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        left_by_the_attempt(directory, "where"),
        "/var/opt/cladenc/harness\n/app\n"
    );
}

#[test]
fn the_harness_of_an_attempt_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        left_by_the_attempt(directory, "model"),
        format!("{TOLD} a-made-up-key\n")
    );
}

#[test]
fn the_grader_of_a_task_is_run_where_the_harness_ran_beside_the_tests_of_its_task() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        kept_with_the_attempt(directory, "grader/stdout"),
        "test.sh\n/app\n"
    );
}

#[test]
fn the_grader_of_a_task_is_told_nothing_of_the_model() {
    let (directory, _output) = &*RAN;
    assert_eq!(
        kept_with_the_attempt(directory, "grader/stderr"),
        "none none none\n"
    );
}

#[test]
fn the_key_is_neither_printed_nor_written_into_the_journal() {
    let (directory, output) = &*RAN;
    let journal = fs::read(directory.path().join("outputs/journal.jsonl")).unwrap();
    let shown = [
        output.stdout.as_slice(),
        output.stderr.as_slice(),
        journal.as_slice(),
    ]
    .concat();
    assert!(!String::from_utf8(shown).unwrap().contains("a-made-up-key"));
}

#[test]
fn the_journal_names_the_file_the_key_is_read_from() {
    let (directory, _output) = &*RAN;
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
    let named = format!(
        "\"model_key_file\":\"{}\"",
        directory.path().join("model-key").display()
    );
    assert!(journal.contains(&named));
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
fn the_harness_of_an_attempt_is_given_no_key_that_no_file_holds_either() {
    let directory = &*RAN_WITH_NO_FILE_OF_A_KEY_AND_A_KEY_IN_THE_ENVIRONMENT;
    assert_eq!(
        left_by_the_attempt(directory, "model"),
        format!("{TOLD} none\n")
    );
}
