use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const SUB_COMMANDS: [&str; 8] = [
    "run", "exec", "build", "rm", "cp", "diff", "inspect", "image",
];
const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const GRADED_ON_ITS_REPORT: &str =
    "artifacts = [\"/app/report.txt\"]\n\n[verifier]\nenvironment_mode = \"separate\"\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const READ_THE_REPORT: &str = "cat /app/report.txt; grep -q found /app/report.txt && echo 1 > \
                               /logs/verifier/reward.txt";
const HOLDS_ITS_TESTS: &str = "COPY . /tests/\n";
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

fn graded_on_its_report() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), WRITE_A_REPORT, GRADED_ON_ITS_REPORT);
    test_inputs::grader(directory.path(), "t2", READ_THE_REPORT);
    start.output().unwrap();
    directory
}

fn with_a_grader_that_is_never_ready() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let output = start_retrying(directory.path(), "exit 0", NEVER_READY, "2")
        .output()
        .unwrap();
    (directory, output)
}

fn resumed_once_the_grader_is_ready() -> Output {
    let (directory, _) = with_a_grader_that_is_never_ready();
    let outputs = directory.path().join("outputs");
    fs::write(outputs.join("benchmarks/benchmark/t2/task.toml"), READY).unwrap();
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!("--outputs-dir={}", outputs.display()))
        .output()
        .unwrap()
}

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

fn arguments_of_a_docker_that_records_them(
    task_toml_of_t2: &str,
    change: impl FnOnce(&Path),
) -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    let mut start = start(directory.path(), "exit 0", task_toml_of_t2);
    change(&directory.path().join("benchmark/t2"));
    start
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    (directory, recorded.lines().map(str::to_owned).collect())
}

fn arguments_with_a_grader_built_from_its_tests() -> (TempDir, Vec<String>) {
    arguments_of_a_docker_that_records_them(SEPARATE, |task| {
        fs::write(
            task.join("tests/Dockerfile"),
            format!("FROM {}\n{HOLDS_ITS_TESTS}", test_inputs::IMAGE),
        )
        .unwrap();
    })
}

fn calls(arguments: &[String], sub_command: &str) -> Vec<Vec<String>> {
    arguments
        .iter()
        .enumerate()
        .filter(|(_, argument)| *argument == sub_command)
        .map(|(place, _)| {
            arguments[place + 1..]
                .iter()
                .take_while(|argument| !SUB_COMMANDS.contains(&argument.as_str()))
                .cloned()
                .collect()
        })
        .collect()
}

fn followed_by(call: &[String], flag: &str) -> String {
    call.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .unwrap_or_default()
}

fn containers_of_t2(arguments: &[String]) -> Vec<Vec<String>> {
    calls(arguments, "run")
        .into_iter()
        .filter(|call| followed_by(call, "--detach").ends_with(":1"))
        .collect()
}

fn place_of(arguments: &[String], sequence: &[&str]) -> Option<usize> {
    arguments
        .windows(sequence.len())
        .position(|found| found == sequence)
}

#[test]
fn a_harness_completes_a_task_whose_grader_runs_separately_by_the_path_that_was_kept() {
    assert_eq!(
        kept_of_the_attempt_at_t2(&graded_on_its_report(), "result"),
        "completed\n"
    );
}

#[test]
fn a_grader_that_runs_separately_reads_the_path_that_was_kept_at_the_path_the_task_lists() {
    assert_eq!(
        kept_of_the_attempt_at_t2(&graded_on_its_report(), "grader/stdout"),
        "found\n"
    );
}

#[test]
fn a_task_whose_grader_runs_separately_with_a_compose_file_in_its_tests_exits_with_2() {
    let (_directory, output) = started_after(SEPARATE, |task| {
        fs::write(task.join("tests/docker-compose.yaml"), "services: {}\n").unwrap();
    });
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_task_whose_grader_runs_separately_with_a_compose_file_in_its_tests_is_reported() {
    let (_directory, output) = started_after(SEPARATE, |task| {
        fs::write(task.join("tests/docker-compose.yaml"), "services: {}\n").unwrap();
    });
    assert!(said(&output).ends_with(
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
    let (_directory, output) = started(&format!(
        "[verifier.environment]\ndocker_image = \"{NOT_ON_THE_MACHINE}\"\n"
    ));
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_grader_whose_image_is_not_on_the_machine_is_reported_last_with_the_task_and_the_image() {
    let (_directory, output) = started(&format!(
        "[verifier.environment]\ndocker_image = \"{NOT_ON_THE_MACHINE}\"\n"
    ));
    assert!(said_last(&output).starts_with(
        "cladenc: the environment of the grader of task benchmark/t2 is not on the machine: its \
         task.toml names the image cladenc-test-not-on-the-machine:0, and cladenc fetches no \
         image: "
    ));
}

#[test]
fn a_grader_whose_environment_is_not_built_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(
        with_tests_built_with("RUN exit 3\n").1.status.code(),
        Some(2)
    );
}

#[test]
fn a_grader_whose_environment_is_not_built_is_reported_last_with_its_task() {
    assert!(
        said_last(&with_tests_built_with("RUN exit 3\n").1).starts_with(
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
fn docker_is_asked_to_build_the_environment_of_a_grader_from_the_tests_of_its_task() {
    let (directory, arguments) = arguments_with_a_grader_built_from_its_tests();
    let outputs = directory.path().join("outputs");
    let build = calls(&arguments, "build")
        .into_iter()
        .find(|call| followed_by(call, "--tag").ends_with(":1-grader"))
        .unwrap_or_default();
    assert_eq!(
        build.get(2..).unwrap_or_default(),
        [
            "--label".to_owned(),
            "cladenc.task=benchmark/t2".to_owned(),
            "--label".to_owned(),
            format!(
                "cladenc.directed-evolution={}",
                fs::canonicalize(&outputs).unwrap().display()
            ),
            outputs
                .join("benchmarks/benchmark/t2/tests")
                .display()
                .to_string(),
        ]
    );
}

#[test]
fn docker_is_asked_to_remove_the_environment_of_a_grader_with_the_environments() {
    let (_directory, arguments) = arguments_with_a_grader_built_from_its_tests();
    assert!(calls(&arguments, "rm").iter().any(|call| {
        call.iter().any(|name| name.ends_with(":1"))
            && call.iter().any(|name| name.ends_with(":1-grader"))
    }));
}

#[test]
fn the_container_of_a_grader_built_from_the_tests_is_a_container_of_that_image() {
    let (_directory, arguments) = arguments_with_a_grader_built_from_its_tests();
    assert!(
        calls(&arguments, "run")
            .iter()
            .any(|call| followed_by(call, "--detach").ends_with(":1-grader"))
    );
}

#[test]
fn the_container_of_the_harness_is_removed_before_that_of_the_grader_is_brought_up() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them(SEPARATE, |_| ());
    let containers = containers_of_t2(&arguments);
    let harness = followed_by(&containers[0], "--name");
    let grader = followed_by(&containers[1], "--name");
    let removed = place_of(&arguments, &["rm", "--force", harness.as_str()]);
    let brought_up = place_of(&arguments, &["--name", grader.as_str()]);
    assert!(
        removed
            .zip(brought_up)
            .is_some_and(|(removed, brought_up)| removed < brought_up)
    );
}

#[test]
fn the_container_of_a_grader_stays_for_the_time_limit_of_the_grader_and_ten_minutes_more() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them(
        "[verifier]\nenvironment_mode = \"separate\"\ntimeout_sec = 20\n",
        |_| (),
    );
    assert_eq!(
        containers_of_t2(&arguments)[1].last().map(String::as_str),
        Some("620")
    );
}

#[test]
fn a_grader_that_is_never_ready_stops_cladenc_start_with_exit_status_1() {
    assert_eq!(with_a_grader_that_is_never_ready().1.status.code(), Some(1));
}

#[test]
fn a_grader_that_is_never_ready_is_reported_last_with_all_that_says_what_happened() {
    let (directory, output) = with_a_grader_that_is_never_ready();
    assert_eq!(
        said_last(&output),
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
    let said = said(&with_a_grader_that_is_never_ready().1);
    let retries = [
        " in a new container after 0 seconds, retry 1 of 2\n",
        " in a new container after 0 seconds, retry 2 of 2\n",
    ]
    .map(|retry| said.contains(retry));
    assert_eq!(retries, [true, true]);
}

#[test]
fn cladenc_start_keeps_no_result_of_an_attempt_whose_grader_was_never_run() {
    let (directory, _) = with_a_grader_that_is_never_ready();
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
        said(&resumed_once_the_grader_is_ready())
            .contains("cladenc: started again the attempt of harness 0 on task benchmark/t2\n")
    );
}

#[test]
fn cladenc_resume_ends_a_directed_evolution_whose_grader_is_ready_at_last_with_exit_status_0() {
    assert_eq!(resumed_once_the_grader_is_ready().status.code(), Some(0));
}
