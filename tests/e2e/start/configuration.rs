use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const NEVER_SET: &str =
    "[environment.env]\nKEY = \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET}\"\n";

fn start(directory: &Path, task_toml_of_t2: &str) -> Command {
    let options = [
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
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    fs::write(directory.join("benchmark/t2/task.toml"), task_toml_of_t2).unwrap();
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn started(task_toml_of_t2: &str) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path(), task_toml_of_t2).output().unwrap();
    (directory, output)
}

static WITH_A_VARIABLE_THAT_IS_NOT_SET: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| started(NEVER_SET));

static WITH_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| started(&in_an_image(NOT_ON_THE_MACHINE)));

static WITH_AN_IMAGE_THAT_IS_ON_THE_MACHINE: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| started(&in_an_image(test_inputs::IMAGE)));

static WITH_TOO_MANY_CPUS: LazyLock<(TempDir, Output)> = LazyLock::new(|| started(TOO_MANY_CPUS));

fn said(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn in_an_image(image: &str) -> String {
    format!("[environment]\ndocker_image = \"{image}\"\n")
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(WITH_A_VARIABLE_THAT_IS_NOT_SET.1.status.code(), Some(2));
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_is_reported_with_the_task_and_the_variable() {
    assert!(said(&WITH_A_VARIABLE_THAT_IS_NOT_SET.1).ends_with(
        "benchmark: task t2 holds a task.toml that gives KEY under environment.env the value of \
         CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET, which is not set where cladenc runs\n"
    ));
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_leaves_no_outputs_directory() {
    let (directory, _output) = &*WITH_A_VARIABLE_THAT_IS_NOT_SET;
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_task_whose_task_toml_is_not_valid_is_reported_with_the_line_it_fails_at() {
    assert!(
        said(&started("[agent]\nuser = \"worker\"\n\n[environment]\nworkdir = 3\n").1).ends_with(
            "benchmark: task t2 holds a task.toml that is not valid at line 5: invalid type: \
             integer `3`, expected a string\n"
        )
    );
}

#[test]
fn a_task_whose_health_check_cannot_be_obeyed_is_reported_with_the_task_and_the_setting() {
    assert!(
        said(&started("[environment.healthcheck]\ncommand = \"true\"\nretries = 0\n").1).ends_with(
            "benchmark: task t2 holds a task.toml that gives environment.healthcheck.retries a \
             value that is not a number above 0\n"
        )
    );
}

#[test]
fn a_path_that_a_task_lists_under_artifacts_is_kept_with_the_attempt() {
    let (directory, _output) = started("artifacts = [\"/app/work.txt\"]\n");
    let attempt = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t2");
    assert_eq!(
        fs::read_to_string(attempt.join("artifacts/app/work.txt")).unwrap(),
        "to do\n"
    );
}

#[test]
fn a_task_that_lists_a_path_that_steps_back_under_artifacts_is_reported_with_the_path() {
    assert!(
        said(&started("artifacts = [\"/app/../etc\"]\n").1).ends_with(
            "benchmark: task t2 holds a task.toml that lists the path `/app/../etc` under \
             artifacts, and a path that is kept holds no `..`\n"
        )
    );
}

#[test]
fn a_task_that_lists_a_path_of_another_container_under_artifacts_is_reported_with_the_task() {
    assert!(
        said(&started("[[artifacts]]\nsource = \"/var/log/api\"\nservice = \"api\"\n").1)
            .ends_with(
                "benchmark: task t2 holds a task.toml that asks for a path of a container other \
                 than that of the harness, which cladenc has no way to give\n"
            )
    );
}

static WITH_NO_TASK_TOML_IN_T2: LazyLock<(TempDir, Output)> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start(directory.path(), "");
    fs::remove_file(directory.path().join("benchmark/t2/task.toml")).unwrap();
    let output = cladenc.output().unwrap();
    (directory, output)
});

#[test]
fn a_task_that_holds_no_task_toml_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(WITH_NO_TASK_TOML_IN_T2.1.status.code(), Some(2));
}

#[test]
fn a_task_that_holds_no_task_toml_is_reported_with_the_task() {
    assert!(
        said(&WITH_NO_TASK_TOML_IN_T2.1)
            .ends_with("benchmark: task t2 holds no file named task.toml\n")
    );
}

#[test]
fn a_task_that_holds_no_task_toml_leaves_no_outputs_directory() {
    let (directory, _output) = &*WITH_NO_TASK_TOML_IN_T2;
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_task_whose_task_toml_is_not_valid_in_what_cladenc_does_not_obey_is_reported_with_its_line() {
    assert!(
        said(&started("[agent]\nuser = \"worker\"\nnetwork_mode = \"private\"\n").1).ends_with(
            "benchmark: task t2 holds a task.toml that is not valid at line 3: invalid value: \
             string \"private\", expected one of no-network, public, allowlist\n"
        )
    );
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(
        WITH_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE.1.status.code(),
        Some(2)
    );
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_is_reported_last_with_the_task_and_the_image() {
    assert!(
        said(&WITH_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE.1)
            .lines()
            .last()
            .unwrap_or_default()
            .starts_with(
                "cladenc: the environment of task benchmark/t2 is not on the machine: its \
                 task.toml names the image cladenc-test-not-on-the-machine:0, and cladenc \
                 fetches no image: "
            )
    );
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_leaves_no_journal() {
    let (directory, _output) = &*WITH_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE;
    assert!(!directory.path().join("outputs/journal.jsonl").exists());
}

#[test]
fn a_task_whose_image_is_on_the_machine_is_said_to_be_taken_as_its_environment() {
    assert!(said(&WITH_AN_IMAGE_THAT_IS_ON_THE_MACHINE.1).contains(
        "cladenc: taking the image debian:stable-slim as the environment of task benchmark/t2, \
         as its task.toml says\n"
    ));
}

#[test]
fn a_directed_evolution_with_a_task_whose_image_is_on_the_machine_runs_to_its_end() {
    assert_eq!(
        WITH_AN_IMAGE_THAT_IS_ON_THE_MACHINE.1.status.code(),
        Some(0)
    );
}

const TOO_MANY_CPUS: &str = "[environment]\ncpus = 100000\n";

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(WITH_TOO_MANY_CPUS.1.status.code(), Some(2));
}

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_is_reported_last_with_what_it_asks_for() {
    assert!(
        said(&WITH_TOO_MANY_CPUS.1)
            .lines()
            .last()
            .unwrap_or_default()
            .starts_with(
                "cladenc: task benchmark/t2 asks for more than the machine has: 100000 under \
                 cpus in its task.toml, where Docker has "
            )
    );
}
