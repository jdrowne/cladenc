use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const SUB_COMMANDS: [&str; 8] = [
    "run", "exec", "build", "rm", "cp", "diff", "inspect", "image",
];
const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const NEVER_SET: &str =
    "[environment.env]\nKEY = \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET}\"\n";
const OBEYED: &str = "[agent]\nuser = 1000\ntimeout_sec = 30\n\n[verifier]\nuser = \
                      2000\ntimeout_sec = 20.0\n\n[verifier.env]\nJUDGE = \
                      \"${CLADENC_TEST_TOKEN}\"\n\n[environment]\nworkdir = \
                      \"/srv/elsewhere\"\n\n[environment.env]\nAPI_TOKEN = \
                      \"${CLADENC_TEST_TOKEN}\"\n";

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

fn said(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn in_an_image(image: &str) -> String {
    format!("[environment]\ndocker_image = \"{image}\"\n")
}

fn arguments_of_a_docker_that_records_them() -> Vec<String> {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), OBEYED)
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .env("CLADENC_TEST_TOKEN", "a-made-up-token")
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    recorded.lines().map(str::to_owned).collect()
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

fn containers_of_the_attempts(arguments: &[String]) -> Vec<Vec<String>> {
    calls(arguments, "run")
        .into_iter()
        .filter(|call| call.iter().any(|argument| argument == "--detach"))
        .collect()
}

fn holds(call: &[String], pair: [&str; 2]) -> bool {
    call.windows(2).any(|found| found == pair)
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(started(NEVER_SET).1.status.code(), Some(2));
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_is_reported_with_the_task_and_the_variable() {
    assert!(said(&started(NEVER_SET).1).ends_with(
        "benchmark: task t2 holds a task.toml that gives KEY under environment.env the value of \
         CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET, which is not set where cladenc runs\n"
    ));
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_leaves_no_outputs_directory() {
    let (directory, _output) = started(NEVER_SET);
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

fn started_with_no_task_toml_in_t2() -> (TempDir, Output) {
    let directory = test_inputs::temporary_directory();
    let mut cladenc = start(directory.path(), "");
    fs::remove_file(directory.path().join("benchmark/t2/task.toml")).unwrap();
    let output = cladenc.output().unwrap();
    (directory, output)
}

#[test]
fn a_task_that_holds_no_task_toml_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(started_with_no_task_toml_in_t2().1.status.code(), Some(2));
}

#[test]
fn a_task_that_holds_no_task_toml_is_reported_with_the_task() {
    assert!(
        said(&started_with_no_task_toml_in_t2().1)
            .ends_with("benchmark: task t2 holds no file named task.toml\n")
    );
}

#[test]
fn a_task_that_holds_no_task_toml_leaves_no_outputs_directory() {
    let (directory, _output) = started_with_no_task_toml_in_t2();
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
        started(&in_an_image(NOT_ON_THE_MACHINE)).1.status.code(),
        Some(2)
    );
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_is_reported_last_with_the_task_and_the_image() {
    assert!(
        said(&started(&in_an_image(NOT_ON_THE_MACHINE)).1)
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
    let (directory, _output) = started(&in_an_image(NOT_ON_THE_MACHINE));
    assert!(!directory.path().join("outputs/journal.jsonl").exists());
}

#[test]
fn a_task_whose_image_is_on_the_machine_is_said_to_be_taken_as_its_environment() {
    assert!(said(&started(&in_an_image(test_inputs::IMAGE)).1).contains(
        "cladenc: taking the image debian:stable-slim as the environment of task benchmark/t2, \
         as its task.toml says\n"
    ));
}

#[test]
fn a_directed_evolution_with_a_task_whose_image_is_on_the_machine_runs_to_its_end() {
    assert_eq!(
        started(&in_an_image(test_inputs::IMAGE)).1.status.code(),
        Some(0)
    );
}

#[test]
fn docker_is_asked_to_build_no_environment_for_a_task_that_names_its_image() {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), &in_an_image("an-image-of-the-task:7"))
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    assert_eq!(
        recorded
            .lines()
            .filter(|line| line.starts_with("cladenc.task="))
            .collect::<Vec<_>>(),
        ["cladenc.task=benchmark/t1", "cladenc.task=benchmark/t3"]
    );
}

#[test]
fn the_value_of_a_variable_of_a_task_is_on_no_command_line_of_docker() {
    assert!(
        !arguments_of_a_docker_that_records_them()
            .concat()
            .contains("a-made-up-token")
    );
}

#[test]
fn the_container_of_an_attempt_is_started_in_the_working_directory_the_task_names() {
    let arguments = arguments_of_a_docker_that_records_them();
    assert_eq!(
        containers_of_the_attempts(&arguments)
            .iter()
            .map(|call| holds(call, ["--workdir", "/srv/elsewhere"]))
            .collect::<Vec<_>>(),
        [false, true, false]
    );
}

#[test]
fn docker_reads_the_variables_of_the_container_of_an_attempt_from_its_standard_input() {
    let arguments = arguments_of_a_docker_that_records_them();
    assert_eq!(
        containers_of_the_attempts(&arguments)
            .iter()
            .map(|call| holds(call, ["--env-file", "/dev/stdin"]))
            .collect::<Vec<_>>(),
        [true, true, true]
    );
}

#[test]
fn the_harness_and_the_grader_of_a_task_are_started_as_the_users_the_task_names() {
    let arguments = arguments_of_a_docker_that_records_them();
    let users: Vec<String> = calls(&arguments, "exec")
        .iter()
        .filter(|call| {
            call.iter()
                .any(|argument| argument == "exit 0" || argument == "/tests/test.sh")
        })
        .map(|call| {
            call.iter()
                .skip_while(|argument| *argument != "--user")
                .nth(1)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(users, ["0", "0", "1000", "2000", "0", "0"]);
}

fn last_arguments_of(calls: &[Vec<String>], command: &str) -> Vec<String> {
    calls
        .iter()
        .filter(|call| call.iter().any(|argument| argument == command))
        .map(|call| call.last().cloned().unwrap_or_default())
        .collect()
}

#[test]
fn the_container_of_an_attempt_sleeps_for_both_time_limits_of_its_task_and_ten_minutes_more() {
    let arguments = arguments_of_a_docker_that_records_them();
    assert_eq!(
        last_arguments_of(&containers_of_the_attempts(&arguments), "sleep"),
        ["4294968495", "650", "4294968495"]
    );
}

#[test]
fn the_timer_of_a_harness_is_given_the_time_limit_its_task_gives_it_and_five_seconds_more() {
    let arguments = arguments_of_a_docker_that_records_them();
    assert_eq!(
        last_arguments_of(&calls(&arguments, "exec"), "exit 0"),
        ["4294967300", "35", "4294967300"]
    );
}

#[test]
fn the_timer_of_a_grader_is_given_the_time_limit_its_task_gives_it_and_five_seconds_more() {
    let arguments = arguments_of_a_docker_that_records_them();
    assert_eq!(
        last_arguments_of(&calls(&arguments, "exec"), "/tests/test.sh"),
        ["605", "25", "605"]
    );
}

const TOO_MANY_CPUS: &str = "[environment]\ncpus = 100000\n";

fn said_with_a_docker_that_says_nothing_of_the_machine() -> (Option<i32>, String) {
    let directory = test_inputs::temporary_directory();
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    let output = start(directory.path(), "[environment]\ncpus = 1\n")
        .env("PATH", path)
        .env(
            "CLADENC_TEST_ARGUMENTS_OF_DOCKER",
            directory.path().join("arguments-of-docker"),
        )
        .output()
        .unwrap();
    (output.status.code(), said(&output))
}

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(started(TOO_MANY_CPUS).1.status.code(), Some(2));
}

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_is_reported_last_with_what_it_asks_for() {
    assert!(
        said(&started(TOO_MANY_CPUS).1)
            .lines()
            .last()
            .unwrap_or_default()
            .starts_with(
                "cladenc: task benchmark/t2 asks for more than the machine has: 100000 under \
                 cpus in its task.toml, where Docker has "
            )
    );
}

#[test]
fn a_docker_that_does_not_say_what_the_machine_has_stops_cladenc_start_with_exit_status_2() {
    assert_eq!(
        said_with_a_docker_that_says_nothing_of_the_machine().0,
        Some(2)
    );
}

#[test]
fn a_docker_that_does_not_say_what_the_machine_has_is_reported_last() {
    assert!(
        said_with_a_docker_that_says_nothing_of_the_machine()
            .1
            .ends_with(
                "cladenc: what the machine has cannot be told: Docker did not say how many CPUs \
                 and how much memory it has\n"
            )
    );
}
