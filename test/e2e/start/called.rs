use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const TOLD: &str = "a-model http://models.example/v1";
const TIMER: &str = r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) & wait $run"#;
const SAY_WHAT_THE_HARNESS_IS_GIVEN: &str = "echo \"$CLADENC_HARNESS\" > where; pwd >> where; \
     echo \"$CLADENC_MODEL $CLADENC_MODEL_URL ${CLADENC_MODEL_KEY-none}\" > model";
const SAY_WHAT_THE_CHECK_IS_GIVEN: &str = "echo \"$CLADENC_TASK\" > where-checked; pwd >> where-checked; \
     echo \"${CLADENC_MODEL-none} ${CLADENC_MODEL_URL-none} ${CLADENC_MODEL_KEY-none}\" > \
     model-of-the-check";

fn start(directory: &Path, key_file: Option<PathBuf>) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            SAY_WHAT_THE_HARNESS_IS_GIVEN.into(),
        ),
        ("--run-dir", directory.join("run")),
        ("--budget", "1".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "1".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .into_iter()
    .chain(key_file.map(|file| ("--model-key-file", file)))
    .map(|(option, value)| format!("{option}={}", value.display()));
    test_inputs::check(directory, "t1", SAY_WHAT_THE_CHECK_IS_GIVEN);
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn ran() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    let output = start(directory.path(), Some(key_file)).output().unwrap();
    (directory, output)
}

fn ran_with_no_file_of_a_key_and_a_key_in_the_environment() -> TempDir {
    let directory = test_inputs::temporary_directory();
    start(directory.path(), None)
        .env("CLADENC_MODEL_KEY", "a-key-from-elsewhere")
        .output()
        .unwrap();
    directory
}

fn ran_with_a_docker_that_records_its_arguments() -> String {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), Some(key_file))
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    fs::read_to_string(arguments).unwrap()
}

fn kept_by_the_harness(directory: &TempDir, file: &str) -> String {
    fs::read_to_string(directory.path().join("run/harnesses/1").join(file)).unwrap()
}

fn left_by_the_evaluation(directory: &TempDir, file: &str) -> String {
    let work = fs::read_dir(directory.path().join("run/evaluations"))
        .unwrap()
        .map(|harness| harness.unwrap().path().join("benchmark/t1/work"))
        .find(|work| work.is_dir())
        .unwrap();
    fs::read_to_string(work.join(file)).unwrap()
}

#[test]
fn a_harness_is_started_in_a_container_that_holds_its_source_code_and_the_copy_to_improve() {
    let (directory, _output) = ran();
    assert_eq!(
        kept_by_the_harness(&directory, "where"),
        "/harness\n/work\n"
    );
}

#[test]
fn a_harness_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = ran();
    assert_eq!(
        kept_by_the_harness(&directory, "model"),
        format!("{TOLD} a-made-up-key\n")
    );
}

#[test]
fn the_harness_of_an_evaluation_is_started_on_the_files_of_the_task() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_evaluation(&directory, "where"),
        "/harness\n/work\n"
    );
}

#[test]
fn the_harness_of_an_evaluation_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_evaluation(&directory, "model"),
        format!("{TOLD} a-made-up-key\n")
    );
}

#[test]
fn the_check_of_a_task_is_run_on_what_the_harness_left_and_told_where_its_task_is() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_evaluation(&directory, "where-checked"),
        "/task\n/work\n"
    );
}

#[test]
fn the_check_of_a_task_is_told_nothing_of_the_model() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_evaluation(&directory, "model-of-the-check"),
        "none none none\n"
    );
}

#[test]
fn the_key_is_neither_printed_nor_written_into_the_journal() {
    let (directory, output) = ran();
    let journal = fs::read(directory.path().join("run/journal.jsonl")).unwrap();
    let shown = [output.stdout, output.stderr, journal].concat();
    assert!(!String::from_utf8(shown).unwrap().contains("a-made-up-key"));
}

#[test]
fn the_journal_names_the_file_the_key_is_read_from() {
    let (directory, _output) = ran();
    let journal = fs::read_to_string(directory.path().join("run/journal.jsonl")).unwrap();
    let named = format!(
        "\"model_key_file\":\"{}\"",
        directory.path().join("model-key").display()
    );
    assert!(journal.contains(&named));
}

#[test]
fn a_harness_is_given_no_key_that_no_file_holds_though_the_environment_of_cladenc_holds_one() {
    let directory = ran_with_no_file_of_a_key_and_a_key_in_the_environment();
    assert_eq!(
        kept_by_the_harness(&directory, "model"),
        format!("{TOLD} none\n")
    );
}

#[test]
fn the_harness_of_an_evaluation_is_given_no_key_that_no_file_holds_either() {
    let directory = ran_with_no_file_of_a_key_and_a_key_in_the_environment();
    assert_eq!(
        left_by_the_evaluation(&directory, "model"),
        format!("{TOLD} none\n")
    );
}

#[test]
fn docker_is_given_the_name_of_the_variable_that_holds_the_key() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains("--env\nCLADENC_MODEL_KEY\n"));
}

#[test]
fn the_key_is_on_no_command_line_of_docker() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(!arguments.contains("a-made-up-key"));
}

#[test]
fn docker_is_told_never_to_fetch_the_image_of_a_container() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    let containers = arguments.lines().filter(|line| *line == "run").count();
    assert_eq!(
        arguments.matches("run\n--rm\n--pull\nnever\n").count(),
        containers.max(1)
    );
}

#[test]
fn every_container_is_started_so_that_it_can_name_the_machine_cladenc_runs_on() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains("--add-host\nhost.docker.internal:host-gateway\n"));
}

#[test]
fn every_container_is_started_under_a_timer_that_ends_its_command() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    let containers = arguments.lines().filter(|line| *line == "run").count();
    assert_eq!(
        arguments.matches(&format!("\nsh\n-c\n{TIMER}\n")).count(),
        containers.max(1)
    );
}

#[test]
fn the_timer_of_a_harness_is_given_its_command_and_five_seconds_more_than_the_time_limit() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains(&format!("{TIMER}\n{SAY_WHAT_THE_HARNESS_IS_GIVEN}\n35\n")));
}

#[test]
fn the_timer_of_a_check_is_given_the_check_of_the_task() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains(&format!("{TIMER}\n/task/check\n")));
}
