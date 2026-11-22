use std::fs;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const AS_ROOT: &str = "WORKDIR /app\nCOPY workspace .\n";
const AS_A_USER: &str = "RUN mkdir /app && chown 1000:1000 /app\nUSER 1000\nWORKDIR /app\n\
                         COPY --chown=1000:1000 workspace .\n";
const HOLDS_LOGS: &str =
    "RUN mkdir /logs && echo held > /logs/held\nWORKDIR /app\nCOPY workspace .\n";
const HOLDS_TESTS: &str =
    "RUN mkdir /tests && echo held > /tests/held\nWORKDIR /app\nCOPY workspace .\n";
const SAVE_A_REPORT: &str = "echo found > /logs/artifacts/report.json";
const PLANT_WHERE_TESTS_POINTS: &str = "mkdir -p /var/opt/cladenc/harbor/tests && echo planted > \
                                        /var/opt/cladenc/harbor/tests/planted.sh";

struct Attempted {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

fn attempted_in(steps: &str, run_command: &str, grader: &str) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", grader);
    fs::write(
        outputs_directory.task(TASK).join("environment/Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let image = test_inputs::image();
    let stop = AtomicBool::new(false);
    containers::build_environment(
        &outputs_directory,
        TASK,
        &image.name,
        Duration::from_secs(600),
        &stop,
    )
    .unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
    };
    let environment = containers::Environment {
        image: image.name.clone(),
        configuration: TaskConfiguration {
            harness_time_limit: Some(Duration::from_secs(30)),
            grader_time_limit: Duration::from_secs(30),
            ..TaskConfiguration::default()
        },
        grader: None,
    };
    let result = containers::run_attempt(
        &outputs_directory,
        &launch,
        containers::Attempt {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &stop,
        &|_| (),
    );
    Attempted {
        _directory: directory,
        _built: image,
        outputs_directory,
        result,
    }
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
}

fn printed_by_the_harness_in(steps: &str, run_command: &str) -> String {
    kept(&attempted_in(steps, run_command, FINISHED), "stdout")
}

fn printed_by_the_harness(run_command: &str) -> String {
    printed_by_the_harness_in(AS_ROOT, run_command)
}

fn printed_by_the_grader_in(steps: &str, run_command: &str, grader: &str) -> String {
    kept(&attempted_in(steps, run_command, grader), "grader/stdout")
}

fn printed_by_the_grader(grader: &str) -> String {
    printed_by_the_grader_in(AS_ROOT, FINISH, grader)
}

fn result_of(attempted: &Attempted) -> Option<AttemptResult> {
    attempted.result.as_ref().ok().map(|ended| ended.result)
}

#[test]
fn the_harness_finds_logs_a_symbolic_link_to_a_directory_under_var_log() {
    assert_eq!(
        printed_by_the_harness("readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn the_harness_finds_under_logs_an_empty_directory_named_artifacts_and_nothing_else() {
    assert_eq!(
        printed_by_the_harness("find /var/log/cladenc/harbor"),
        "/var/log/cladenc/harbor\n/var/log/cladenc/harbor/artifacts\n"
    );
}

#[test]
fn logs_artifacts_belongs_to_the_user_the_harness_runs_as() {
    assert_eq!(
        printed_by_the_harness_in(AS_A_USER, "stat -c %u /logs/artifacts"),
        "1000\n"
    );
}

#[test]
fn what_a_harness_that_does_not_run_as_root_saved_in_logs_artifacts_is_kept_with_the_attempt() {
    let attempted = attempted_in(AS_A_USER, SAVE_A_REPORT, FINISHED);
    assert_eq!(
        kept(&attempted, "artifacts/logs/artifacts/report.json"),
        "found\n"
    );
}

#[test]
fn a_file_the_harness_saved_in_logs_artifacts_is_under_var_log() {
    assert_eq!(
        printed_by_the_harness(&format!(
            "{SAVE_A_REPORT}; cat /var/log/cladenc/harbor/artifacts/report.json"
        )),
        "found\n"
    );
}

#[test]
fn an_attempt_whose_harness_saved_nothing_in_logs_artifacts_keeps_no_artifacts() {
    let attempted = attempted_in(AS_ROOT, FINISH, FINISHED);
    assert!(
        !attempted
            .outputs_directory
            .attempt(0, TASK)
            .join("artifacts")
            .exists()
    );
}

#[test]
fn logs_that_the_environment_of_the_task_holds_is_left_as_it_is() {
    assert_eq!(
        printed_by_the_harness_in(HOLDS_LOGS, "cat /logs/held; [ -L /logs ] || echo no link"),
        "held\nno link\n"
    );
}

#[test]
fn logs_artifacts_is_made_in_the_logs_that_the_environment_of_the_task_holds() {
    let attempted = attempted_in(HOLDS_LOGS, SAVE_A_REPORT, FINISHED);
    assert_eq!(
        kept(&attempted, "artifacts/logs/artifacts/report.json"),
        "found\n"
    );
}

#[test]
fn nothing_is_made_under_var_log_where_the_environment_of_the_task_holds_logs() {
    assert_eq!(
        printed_by_the_harness_in(
            HOLDS_LOGS,
            "ls -d /var/log/cladenc 2> /dev/null; echo looked"
        ),
        "looked\n"
    );
}

#[test]
fn the_grader_scores_in_the_logs_that_the_environment_of_the_task_holds() {
    assert_eq!(
        result_of(&attempted_in(HOLDS_LOGS, FINISH, FINISHED)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn the_grader_finds_logs_verifier_under_var_log_beside_logs_artifacts() {
    assert_eq!(
        printed_by_the_grader("find /var/log/cladenc/harbor | sort"),
        "/var/log/cladenc/harbor\n/var/log/cladenc/harbor/artifacts\n\
         /var/log/cladenc/harbor/verifier\n"
    );
}

#[test]
fn logs_verifier_belongs_to_the_user_the_grader_runs_as() {
    assert_eq!(
        printed_by_the_grader_in(
            AS_A_USER,
            FINISH,
            "stat -c %u /var/log/cladenc/harbor/verifier"
        ),
        "1000\n"
    );
}

#[test]
fn the_grader_finds_logs_a_symbolic_link_again_where_the_harness_removed_it() {
    assert_eq!(
        printed_by_the_grader_in(AS_ROOT, "rm /logs", "readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn the_grader_finds_tests_a_symbolic_link_to_a_directory_under_var_opt() {
    assert_eq!(
        printed_by_the_grader("readlink /tests"),
        "/var/opt/cladenc/harbor/tests\n"
    );
}

#[test]
fn the_directory_that_tests_points_to_holds_the_tests_of_the_task_and_nothing_else() {
    assert_eq!(
        printed_by_the_grader("find /var/opt/cladenc/harbor"),
        "/var/opt/cladenc/harbor\n/var/opt/cladenc/harbor/tests\n\
         /var/opt/cladenc/harbor/tests/test.sh\n"
    );
}

#[test]
fn the_tests_of_the_task_belong_to_the_user_the_grader_runs_as() {
    assert_eq!(
        printed_by_the_grader_in(
            AS_A_USER,
            FINISH,
            "stat -c %u /var/opt/cladenc/harbor/tests /tests/test.sh"
        ),
        "1000\n1000\n"
    );
}

#[test]
fn a_file_the_harness_left_where_tests_points_is_not_found_by_the_grader() {
    assert_eq!(
        printed_by_the_grader_in(AS_ROOT, PLANT_WHERE_TESTS_POINTS, "ls -A /tests"),
        "test.sh\n"
    );
}

#[test]
fn what_the_environment_of_the_task_holds_at_tests_is_not_found_by_the_grader() {
    assert_eq!(
        printed_by_the_grader_in(HOLDS_TESTS, FINISH, "ls -A /tests"),
        "test.sh\n"
    );
}

#[test]
fn the_grader_finds_logs_and_tests_at_the_top_of_the_filesystem_as_symbolic_links() {
    assert_eq!(
        printed_by_the_grader("[ -L /logs ] && [ -L /tests ] && echo links"),
        "links\n"
    );
}

#[test]
fn the_container_of_an_attempt_holds_nothing_under_var_lib_that_is_named_after_cladenc() {
    assert_eq!(
        printed_by_the_harness("ls -d /var/lib/cladenc 2> /dev/null; echo looked"),
        "looked\n"
    );
}
