use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const TOLD: &str = "a-model http://models.example/v1";
const TIMER: &str =
    r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) > /dev/null 2>&1 & wait $run"#;
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
    ran_in_with_a_docker_that_records_its_arguments(&test_inputs::temporary_directory())
}

fn ran_in_with_a_docker_that_records_its_arguments(directory: &TempDir) -> String {
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
    fs::read_to_string(directory.path().join("outputs/harnesses/1").join(file)).unwrap()
}

fn kept_with_the_attempt(directory: &TempDir, file: &str) -> String {
    let kept = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t1");
    fs::read_to_string(kept.join(file)).unwrap()
}

fn left_by_the_attempt(directory: &TempDir, file: &str) -> String {
    kept_with_the_attempt(directory, &format!("work/{file}"))
}

fn followed_by<'a>(arguments: &'a str, word: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = arguments.lines().collect();
    lines
        .windows(2)
        .filter(|pair| pair[0] == word)
        .map(|pair| pair[1])
        .collect()
}

#[test]
fn a_harness_is_started_in_a_container_that_holds_its_source_code_and_the_copy_to_improve() {
    let (directory, _output) = ran();
    assert_eq!(
        kept_by_the_harness(&directory, "where"),
        "/var/opt/cladenc/harness\n/var/opt/cladenc/work\n"
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
fn the_harness_of_an_attempt_is_started_in_the_working_directory_of_the_environment_of_its_task() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_attempt(&directory, "where"),
        "/var/opt/cladenc/harness\n/app\n"
    );
}

#[test]
fn the_harness_of_an_attempt_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    let (directory, _output) = ran();
    assert_eq!(
        left_by_the_attempt(&directory, "model"),
        format!("{TOLD} a-made-up-key\n")
    );
}

#[test]
fn the_grader_of_a_task_is_run_where_the_harness_ran_beside_the_tests_of_its_task() {
    let (directory, _output) = ran();
    assert_eq!(
        kept_with_the_attempt(&directory, "grader/stdout"),
        "test.sh\n/app\n"
    );
}

#[test]
fn the_grader_of_a_task_is_told_nothing_of_the_model() {
    let (directory, _output) = ran();
    assert_eq!(
        kept_with_the_attempt(&directory, "grader/stderr"),
        "none none none\n"
    );
}

#[test]
fn the_key_is_neither_printed_nor_written_into_the_journal() {
    let (directory, output) = ran();
    let journal = fs::read(directory.path().join("outputs/journal.jsonl")).unwrap();
    let shown = [output.stdout, output.stderr, journal].concat();
    assert!(!String::from_utf8(shown).unwrap().contains("a-made-up-key"));
}

#[test]
fn the_journal_names_the_file_the_key_is_read_from() {
    let (directory, _output) = ran();
    let journal = fs::read_to_string(directory.path().join("outputs/journal.jsonl")).unwrap();
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
fn the_harness_of_an_attempt_is_given_no_key_that_no_file_holds_either() {
    let directory = ran_with_no_file_of_a_key_and_a_key_in_the_environment();
    assert_eq!(
        left_by_the_attempt(&directory, "model"),
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

fn count_of(arguments: &str, word: &str) -> usize {
    arguments.lines().filter(|line| *line == word).count()
}

#[test]
fn every_command_is_started_under_a_timer_that_ends_it() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    let timed = count_of(&arguments, "run") + count_of(&arguments, "exec");
    let as_root = arguments.matches("exec\n--user\n0\n").count();
    let commands = timed - count_of(&arguments, "--detach") - as_root;
    assert_eq!(
        arguments.matches(&format!("\nsh\n-c\n{TIMER}\n")).count(),
        commands.max(1)
    );
}

#[test]
fn the_container_of_an_attempt_whose_task_gives_no_time_limit_sleeps_without_end() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    let image = followed_by(&arguments, "--detach")[0];
    assert!(arguments.contains(&format!("--detach\n{image}\nsleep\n4294968495\n")));
}

#[test]
fn the_containers_of_the_attempts_are_started_from_the_images_built_from_the_environments() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    let started: BTreeSet<&str> = followed_by(&arguments, "--detach").into_iter().collect();
    let built: BTreeSet<&str> = followed_by(&arguments, "--tag").into_iter().collect();
    assert_eq!(started, built);
}

#[test]
fn the_timer_of_a_harness_is_given_its_command_and_five_seconds_more_than_the_time_limit() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains(&format!("{TIMER}\n{SAY_WHAT_THE_HARNESS_IS_GIVEN}\n35\n")));
}

#[test]
fn the_timer_of_a_grader_is_given_the_grader_of_the_task() {
    let arguments = ran_with_a_docker_that_records_its_arguments();
    assert!(arguments.contains(&format!("{TIMER}\n/tests/test.sh\n")));
}

#[test]
fn every_container_carries_the_label_of_its_directed_evolution() {
    let directory = test_inputs::temporary_directory();
    let arguments = ran_in_with_a_docker_that_records_its_arguments(&directory);
    let outputs = fs::canonicalize(directory.path().join("outputs")).unwrap();
    let label = format!("cladenc.directed-evolution={}", outputs.display());
    let lines: Vec<&str> = arguments.lines().collect();
    let containers = lines.iter().filter(|line| **line == "run").count();
    assert_eq!(
        lines
            .windows(4)
            .filter(|told| told[0] == "--name" && told[2] == "--label" && told[3] == label)
            .count(),
        containers.max(1)
    );
}
