use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const TOLD: &str = "a-model http://models.example/v1";

fn start(directory: &Path, key_file: Option<PathBuf>) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            "echo \"$CLADENC_HARNESS\" > where; pwd >> where; \
             echo \"$CLADENC_MODEL $CLADENC_MODEL_URL ${CLADENC_MODEL_KEY-none}\" > model"
                .into(),
        ),
        ("--evaluator", test_inputs::sample("e2e", "record.sh")),
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

fn output_of_the_evaluation(directory: &TempDir) -> PathBuf {
    fs::read_dir(directory.path().join("run/evaluations"))
        .unwrap()
        .map(|harness| harness.unwrap().path().join("t1/output"))
        .find(|output| output.is_dir())
        .unwrap()
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
fn the_evaluator_is_given_the_harness_the_task_and_the_output() {
    let (directory, _output) = ran();
    let run = directory.path().join("run");
    let evaluated = output_of_the_evaluation(&directory);
    let harness = evaluated
        .strip_prefix(run.join("evaluations"))
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(
        fs::read_to_string(evaluated.join("arguments")).unwrap(),
        format!(
            "{0}/running/evaluation-{harness}-t1/harness\n\
             t1\n\
             {0}/running/evaluation-{harness}-t1/output\n",
            run.display()
        )
    );
}

#[test]
fn the_evaluator_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = ran();
    assert_eq!(
        fs::read_to_string(output_of_the_evaluation(&directory).join("model")).unwrap(),
        format!("{TOLD} a-made-up-key\n")
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
fn the_evaluator_is_given_no_key_that_no_file_holds_though_the_environment_of_cladenc_holds_one() {
    let directory = ran_with_no_file_of_a_key_and_a_key_in_the_environment();
    assert_eq!(
        fs::read_to_string(output_of_the_evaluation(&directory).join("model")).unwrap(),
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
fn every_container_is_started_so_that_it_can_name_the_machine_cladenc_runs_on() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains("--add-host\nhost.docker.internal:host-gateway\n"));
}
