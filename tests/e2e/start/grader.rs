use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const GRADED_ON_ITS_REPORT: &str =
    "artifacts = [\"/app/report.txt\"]\n\n[verifier]\nenvironment_mode = \"separate\"\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const READ_THE_REPORT: &str = "cat /app/report.txt; grep -q found /app/report.txt && echo 1 > \
                               /logs/verifier/reward.txt";
const NEVER_READY: &str = "[verifier.environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
const READY: &str = "[verifier.environment.healthcheck]\ncommand = \"true\"\n";

fn start(directory: &Path, run_command: &str, task_toml_of_t2: &str) -> Command {
    start_retrying(directory, run_command, task_toml_of_t2, "0")
}

fn start_retrying(
    directory: &Path,
    run_command: &str,
    task_toml_of_t2: &str,
    max_retries: &str,
) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", max_retries.into()),
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

fn started_after(task_toml_of_t2: &str, change: impl FnOnce(&Path)) -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), "exit 0", task_toml_of_t2);
    change(&directory.path().join("benchmark/t2"));
    let output = start.output().unwrap();
    (directory, output)
}

fn started(task_toml_of_t2: &str) -> (TempDir, Output) {
    started_after(task_toml_of_t2, |_| ())
}

fn with_tests_built_with(steps: &str) -> (TempDir, Output) {
    started_after(SEPARATE, |task| {
        fs::write(
            task.join("tests/Dockerfile"),
            format!("FROM {}\n{steps}", test_inputs::IMAGE),
        )
        .unwrap();
    })
}

static WITH_A_TASK_GRADED_ON_ITS_REPORT: LazyLock<TempDir> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), WRITE_A_REPORT, GRADED_ON_ITS_REPORT);
    test_inputs::grader(directory.path(), "t2", READ_THE_REPORT);
    start.output().unwrap();
    directory
});

fn with_a_grader_that_is_never_ready() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = start_retrying(directory.path(), "exit 0", NEVER_READY, "2")
        .output()
        .unwrap();
    (directory, output)
}

static RESUMED_ONCE_THE_GRADER_IS_READY: LazyLock<Output> = LazyLock::new(|| {
    let (directory, _) = with_a_grader_that_is_never_ready();
    let outputs = directory.path().join("outputs");
    fs::write(outputs.join("benchmarks/benchmark/t2/task.toml"), READY).unwrap();
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--outputs-dir={}", outputs.display()))
        .output()
        .unwrap()
});

static WITH_A_COMPOSE_FILE_IN_ITS_TESTS: LazyLock<(TempDir, Output)> = LazyLock::new(|| {
    started_after(SEPARATE, |task| {
        fs::write(task.join("tests/docker-compose.yaml"), "services: {}\n").unwrap();
    })
});

static WITH_A_GRADER_WHOSE_IMAGE_IS_NOT_ON_THE_MACHINE: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| {
        started(&format!(
            "[verifier.environment]\ndocker_image = \"{NOT_ON_THE_MACHINE}\"\n"
        ))
    });

static WITH_A_GRADER_WHOSE_ENVIRONMENT_IS_NOT_BUILT: LazyLock<(TempDir, Output)> =
    LazyLock::new(|| with_tests_built_with("RUN exit 3\n"));

static WITH_A_GRADER_THAT_IS_NEVER_READY: LazyLock<(TempDir, Output)> =
    LazyLock::new(with_a_grader_that_is_never_ready);

fn kept_of_the_attempt_at_t2(directory: &TempDir, file: &str) -> String {
    let attempt = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t2");
    fs::read_to_string(attempt.join(file)).unwrap()
}

fn said(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn said_last(output: &Output) -> String {
    said(output).lines().last().unwrap_or_default().to_owned()
}

#[test]
fn a_harness_completes_a_task_whose_grader_runs_separately_by_the_path_that_was_kept() {
    assert_eq!(
        kept_of_the_attempt_at_t2(&WITH_A_TASK_GRADED_ON_ITS_REPORT, "result"),
        "completed\n"
    );
}

#[test]
fn a_grader_that_runs_separately_reads_the_path_that_was_kept_at_the_path_the_task_lists() {
    assert_eq!(
        kept_of_the_attempt_at_t2(&WITH_A_TASK_GRADED_ON_ITS_REPORT, "grader/stdout"),
        "found\n"
    );
}

#[test]
fn a_task_whose_grader_runs_separately_with_a_compose_file_in_its_tests_exits_with_2() {
    let (_directory, output) = &*WITH_A_COMPOSE_FILE_IN_ITS_TESTS;
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_task_whose_grader_runs_separately_with_a_compose_file_in_its_tests_is_reported() {
    let (_directory, output) = &*WITH_A_COMPOSE_FILE_IN_ITS_TESTS;
    assert!(said(output).ends_with(
        "benchmark: task t2 has its grader run in a container of its own and holds a Compose \
         file in its directory named tests, and an environment of more than one container cannot \
         be built\n"
    ));
}

#[test]
fn a_task_that_lists_the_whole_container_for_a_grader_that_runs_separately_is_reported() {
    assert!(
        said(&started(&format!("artifacts = [\"/\"]\n\n{SEPARATE}")).1).ends_with(
            "benchmark: task t2 holds a task.toml that asks for the whole container of the \
             harness in the container of the grader, which cladenc has no way to give\n"
        )
    );
}

#[test]
fn a_grader_whose_image_is_not_on_the_machine_stops_cladenc_start_with_exit_status_2() {
    let (_directory, output) = &*WITH_A_GRADER_WHOSE_IMAGE_IS_NOT_ON_THE_MACHINE;
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_grader_whose_image_is_not_on_the_machine_is_reported_last_with_the_task_and_the_image() {
    let (_directory, output) = &*WITH_A_GRADER_WHOSE_IMAGE_IS_NOT_ON_THE_MACHINE;
    assert!(said_last(output).starts_with(
        "cladenc: the environment of the grader of task benchmark/t2 is not on the machine: its \
         task.toml names the image cladenc-test-not-on-the-machine:0, and cladenc fetches no \
         image: "
    ));
}

#[test]
fn a_grader_whose_environment_is_not_built_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(
        WITH_A_GRADER_WHOSE_ENVIRONMENT_IS_NOT_BUILT.1.status.code(),
        Some(2)
    );
}

#[test]
fn a_grader_whose_environment_is_not_built_is_reported_last_with_its_task() {
    assert!(
        said_last(&WITH_A_GRADER_WHOSE_ENVIRONMENT_IS_NOT_BUILT.1).starts_with(
            "cladenc: the environment of the grader of task benchmark/t2 was not built: its \
             build exited with "
        )
    );
}

#[test]
fn a_task_that_names_the_image_of_its_grader_and_holds_no_tests_is_not_refused() {
    let (_directory, output) = started_after(
        &format!(
            "[verifier.environment]\ndocker_image = \"{}\"\n",
            test_inputs::IMAGE
        ),
        |task| fs::remove_dir_all(task.join("tests")).unwrap(),
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_grader_that_is_never_ready_stops_cladenc_start_with_exit_status_1() {
    assert_eq!(WITH_A_GRADER_THAT_IS_NEVER_READY.1.status.code(), Some(1));
}

#[test]
fn a_grader_that_is_never_ready_is_reported_last_with_all_that_says_what_happened() {
    let (directory, output) = &*WITH_A_GRADER_THAT_IS_NEVER_READY;
    assert_eq!(
        said_last(output),
        format!(
            "cladenc: the directed evolution stopped before its end: the attempt of harness 0 \
             on task benchmark/t2 was not graded: the health check of the container of its \
             grader did not pass in 3 tries, so the grader was not run, and \
             {}/running/attempt-0-benchmark+t2/grader/health-check holds what the health check \
             printed the last time it ran",
            directory.path().join("outputs").display()
        )
    );
}

#[test]
fn cladenc_start_says_each_retry_of_the_health_check_of_a_grader() {
    let said = said(&WITH_A_GRADER_THAT_IS_NEVER_READY.1);
    let retries = [
        " in a new container after 0 seconds, retry 1 of 2\n",
        " in a new container after 0 seconds, retry 2 of 2\n",
    ]
    .map(|retry| said.contains(retry));
    assert_eq!(retries, [true, true]);
}

#[test]
fn cladenc_start_keeps_no_result_of_an_attempt_whose_grader_was_never_run() {
    let (directory, _) = &*WITH_A_GRADER_THAT_IS_NEVER_READY;
    let kept = ["attempts", "holdout"].map(|kept| {
        directory
            .path()
            .join("outputs")
            .join(kept)
            .join("0/benchmark/t2")
            .exists()
    });
    assert_eq!(kept, [false, false]);
}

#[test]
fn cladenc_resume_carries_out_again_an_attempt_whose_grader_was_never_run() {
    assert!(
        said(&RESUMED_ONCE_THE_GRADER_IS_READY)
            .contains("cladenc: started again the attempt of harness 0 on task benchmark/t2\n")
    );
}

#[test]
fn cladenc_resume_ends_a_directed_evolution_whose_grader_is_ready_at_last_with_exit_status_0() {
    assert_eq!(RESUMED_ONCE_THE_GRADER_IS_READY.status.code(), Some(0));
}
