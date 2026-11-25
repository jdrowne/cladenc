use std::fs;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Host, Launch, Program};
use cladenc::file_system::Disk;
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

struct Asked {
    attempted: Attempted,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
}

fn attempted_in(steps: &str, run_command: &str, verifier: &str) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", verifier);
    fs::write(
        outputs_directory.task(TASK).join("environment/Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let image = test_inputs::image();
    let stop = AtomicBool::new(false);
    containers::build_environment(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
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
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let environment = containers::Environment {
        image: image.name.clone(),
        configuration: TaskConfiguration {
            harness_time_limit: Some(Duration::from_secs(30)),
            verifier_time_limit: Duration::from_secs(30),
            ..TaskConfiguration::default()
        },
        verifier: None,
    };
    let result = containers::run_attempt(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
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

fn asked_in(
    steps: &str,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
) -> Asked {
    Asked {
        attempted: attempted_in(
            steps,
            &test_inputs::one_after_another(of_the_harness),
            &test_inputs::one_after_another(of_the_verifier),
        ),
        of_the_harness,
        of_the_verifier,
    }
}

static AS_ROOT_AND_LOOKED_AT: LazyLock<Asked> = LazyLock::new(|| {
    asked_in(
        AS_ROOT,
        &[
            "readlink /logs",
            "find /var/log/cladenc/harbor",
            "ls -d /var/lib/cladenc 2> /dev/null; echo looked",
            FINISH,
        ],
        &[
            "find /var/log/cladenc/harbor | sort",
            "readlink /tests",
            "find /var/opt/cladenc/harbor",
            "[ -L /logs ] && [ -L /tests ] && echo links",
        ],
    )
});

static AS_ROOT_AND_WRITTEN_TO: LazyLock<Asked> = LazyLock::new(|| {
    asked_in(
        AS_ROOT,
        &[
            SAVE_A_REPORT,
            "cat /var/log/cladenc/harbor/artifacts/report.json",
            PLANT_WHERE_TESTS_POINTS,
        ],
        &["ls -A /tests"],
    )
});

static AS_A_USER_AND_LOOKED_AT: LazyLock<Asked> = LazyLock::new(|| {
    asked_in(
        AS_A_USER,
        &["stat -c %u /logs/artifacts", SAVE_A_REPORT, FINISH],
        &[
            "stat -c %u /var/log/cladenc/harbor/verifier",
            "stat -c %u /var/opt/cladenc/harbor/tests /tests/test.sh",
        ],
    )
});

static HOLDING_LOGS_AND_LOOKED_AT: LazyLock<Asked> = LazyLock::new(|| {
    asked_in(
        HOLDS_LOGS,
        &[
            "cat /logs/held; [ -L /logs ] || echo no link",
            "ls -d /var/log/cladenc 2> /dev/null; echo looked",
            SAVE_A_REPORT,
            FINISH,
        ],
        &[FINISHED],
    )
});

fn printed_by_the_harness(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_harness,
        &kept(&asked.attempted, "stdout"),
    )
}

fn printed_by_the_verifier(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_verifier,
        &kept(&asked.attempted, "verifier/stdout"),
    )
}

fn printed_by_the_verifier_in(steps: &str, run_command: &str, verifier: &str) -> String {
    kept(
        &attempted_in(steps, run_command, verifier),
        "verifier/stdout",
    )
}

fn result_of(attempted: &Attempted) -> Option<AttemptResult> {
    attempted.result.as_ref().ok().map(|ended| ended.result)
}

#[test]
fn the_harness_finds_logs_a_symbolic_link_to_a_directory_under_var_log() {
    assert_eq!(
        printed_by_the_harness(&AS_ROOT_AND_LOOKED_AT, "readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn the_harness_finds_under_logs_an_empty_directory_named_artifacts_and_nothing_else() {
    assert_eq!(
        printed_by_the_harness(&AS_ROOT_AND_LOOKED_AT, "find /var/log/cladenc/harbor"),
        "/var/log/cladenc/harbor\n/var/log/cladenc/harbor/artifacts\n"
    );
}

#[test]
fn logs_artifacts_belongs_to_the_user_the_harness_runs_as() {
    assert_eq!(
        printed_by_the_harness(&AS_A_USER_AND_LOOKED_AT, "stat -c %u /logs/artifacts"),
        "1000\n"
    );
}

#[test]
fn what_a_harness_that_does_not_run_as_root_saved_in_logs_artifacts_is_kept_with_the_attempt() {
    assert_eq!(
        kept(
            &AS_A_USER_AND_LOOKED_AT.attempted,
            "artifacts/logs/artifacts/report.json"
        ),
        "found\n"
    );
}

#[test]
fn a_file_the_harness_saved_in_logs_artifacts_is_under_var_log() {
    assert_eq!(
        printed_by_the_harness(
            &AS_ROOT_AND_WRITTEN_TO,
            "cat /var/log/cladenc/harbor/artifacts/report.json"
        ),
        "found\n"
    );
}

#[test]
fn logs_that_the_environment_of_the_task_holds_is_left_as_it_is() {
    assert_eq!(
        printed_by_the_harness(
            &HOLDING_LOGS_AND_LOOKED_AT,
            "cat /logs/held; [ -L /logs ] || echo no link"
        ),
        "held\nno link\n"
    );
}

#[test]
fn logs_artifacts_is_made_in_the_logs_that_the_environment_of_the_task_holds() {
    assert_eq!(
        kept(
            &HOLDING_LOGS_AND_LOOKED_AT.attempted,
            "artifacts/logs/artifacts/report.json"
        ),
        "found\n"
    );
}

#[test]
fn nothing_is_made_under_var_log_where_the_environment_of_the_task_holds_logs() {
    assert_eq!(
        printed_by_the_harness(
            &HOLDING_LOGS_AND_LOOKED_AT,
            "ls -d /var/log/cladenc 2> /dev/null; echo looked"
        ),
        "looked\n"
    );
}

#[test]
fn the_verifier_scores_in_the_logs_that_the_environment_of_the_task_holds() {
    assert_eq!(
        result_of(&HOLDING_LOGS_AND_LOOKED_AT.attempted),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn the_verifier_finds_logs_verifier_under_var_log_beside_logs_artifacts() {
    assert_eq!(
        printed_by_the_verifier(
            &AS_ROOT_AND_LOOKED_AT,
            "find /var/log/cladenc/harbor | sort"
        ),
        "/var/log/cladenc/harbor\n/var/log/cladenc/harbor/artifacts\n\
         /var/log/cladenc/harbor/verifier\n"
    );
}

#[test]
fn logs_verifier_belongs_to_the_user_the_verifier_runs_as() {
    assert_eq!(
        printed_by_the_verifier(
            &AS_A_USER_AND_LOOKED_AT,
            "stat -c %u /var/log/cladenc/harbor/verifier"
        ),
        "1000\n"
    );
}

#[test]
fn the_verifier_finds_logs_a_symbolic_link_again_where_the_harness_removed_it() {
    assert_eq!(
        printed_by_the_verifier_in(AS_ROOT, "rm /logs", "readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn the_verifier_finds_tests_a_symbolic_link_to_a_directory_under_var_opt() {
    assert_eq!(
        printed_by_the_verifier(&AS_ROOT_AND_LOOKED_AT, "readlink /tests"),
        "/var/opt/cladenc/harbor/tests\n"
    );
}

#[test]
fn the_directory_that_tests_points_to_holds_the_tests_of_the_task_and_nothing_else() {
    assert_eq!(
        printed_by_the_verifier(&AS_ROOT_AND_LOOKED_AT, "find /var/opt/cladenc/harbor"),
        "/var/opt/cladenc/harbor\n/var/opt/cladenc/harbor/tests\n\
         /var/opt/cladenc/harbor/tests/test.sh\n"
    );
}

#[test]
fn the_tests_of_the_task_belong_to_the_user_the_verifier_runs_as() {
    assert_eq!(
        printed_by_the_verifier(
            &AS_A_USER_AND_LOOKED_AT,
            "stat -c %u /var/opt/cladenc/harbor/tests /tests/test.sh"
        ),
        "1000\n1000\n"
    );
}

#[test]
fn a_file_the_harness_left_where_tests_points_is_not_found_by_the_verifier() {
    assert_eq!(
        printed_by_the_verifier(&AS_ROOT_AND_WRITTEN_TO, "ls -A /tests"),
        "test.sh\n"
    );
}

#[test]
fn what_the_environment_of_the_task_holds_at_tests_is_not_found_by_the_verifier() {
    assert_eq!(
        printed_by_the_verifier_in(HOLDS_TESTS, FINISH, "ls -A /tests"),
        "test.sh\n"
    );
}

#[test]
fn the_verifier_finds_logs_and_tests_at_the_top_of_the_filesystem_as_symbolic_links() {
    assert_eq!(
        printed_by_the_verifier(
            &AS_ROOT_AND_LOOKED_AT,
            "[ -L /logs ] && [ -L /tests ] && echo links"
        ),
        "links\n"
    );
}

#[test]
fn the_container_of_an_attempt_holds_nothing_under_var_lib_that_is_named_after_cladenc() {
    assert_eq!(
        printed_by_the_harness(
            &AS_ROOT_AND_LOOKED_AT,
            "ls -d /var/lib/cladenc 2> /dev/null; echo looked"
        ),
        "looked\n"
    );
}
