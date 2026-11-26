use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::genealogy::{Cause, TrialResult};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Host, Launch, Program, SandboxError};
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const LEAVE_A_PROCESS_RUNNING: &str =
    "(while true; do echo beat >> beats; sleep 1; done) > /dev/null 2>&1 & sleep 1";
const THE_PROCESS_STILL_RUNS: &str = "before=$(wc -l < beats); sleep 3; \
    [ \"$(wc -l < beats)\" -gt \"$before\" ] && echo 1 > /logs/verifier/reward.txt";
const FINISH_AND_STAY: &str = "echo done > work.txt; sleep 300";
const LEAVE_A_PROCESS_RUNNING_AND_STAY: &str =
    "(while true; do echo beat >> beats; sleep 1; done) > /dev/null 2>&1 & sleep 300";
const THE_PROCESS_NO_LONGER_RUNS: &str = "before=$(wc -l < beats); sleep 3; \
    [ \"$(wc -l < beats)\" -eq \"$before\" ] && echo 1 > /logs/verifier/reward.txt";
const LEAVE_A_DIRECTORY_NO_ONE_CAN_WRITE_IN: &str =
    "mkdir -p locked/inner && touch locked/inner/file && chmod 500 locked/inner locked";
const REMOVE_LOGS_VERIFIER: &str = "rm -rf /logs/verifier";
const LEAVE_WHAT_CANNOT_BE_COPIED_OUT: &str = "cd /logs/verifier && mkdir locked && \
    touch locked/file && chmod 500 locked && echo 1 > reward.txt";
const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const LOOK_FOR_THE_TESTS: &str = "ls -d /tests /var/opt/cladenc/harbor/tests \
    /var/log/cladenc/trials 2> /dev/null";
const TASK: &str = "benchmark/t1";
const AS_A_USER: &str = "RUN mkdir /app && chown 1000:1000 /app\nUSER 1000\nWORKDIR /app\n\
                         COPY --chown=1000:1000 workspace .\n";

struct Trialled {
    _directory: TempDir,
    _built: Option<Image>,
    environment_image: String,
    outputs_directory: OutputsDirectory,
    result: Result<sandboxes::Trialled, SandboxError>,
}

struct Asked {
    trialled: Trialled,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum Environment<'a> {
    OfTheTask,
    BuiltWith(&'a str),
    Named(&'a str),
}

struct Trial<'a> {
    environment: Environment<'a>,
    run_command: &'a str,
    verifier: &'a str,
    key: Option<&'a str>,
    harness_time_limit: Option<u64>,
    unreadable_in_the_harness: bool,
}

impl Default for Trial<'_> {
    fn default() -> Self {
        Self {
            environment: Environment::OfTheTask,
            run_command: FINISH,
            verifier: FINISHED,
            key: None,
            harness_time_limit: Some(30),
            unreadable_in_the_harness: false,
        }
    }
}

fn built(outputs_directory: &OutputsDirectory) -> Image {
    let image = test_inputs::image();
    sandboxes::build_environment(
        Host {
            docker: &Program,
            outputs_directory,
        },
        TASK,
        &image.name,
        Duration::from_secs(600),
        &AtomicBool::new(false),
    )
    .unwrap();
    image
}

fn environment_of(
    outputs_directory: &OutputsDirectory,
    environment: Environment<'_>,
) -> (String, Option<Image>) {
    if let Environment::BuiltWith(steps) = environment {
        fs::write(
            outputs_directory.task(TASK).join("environment/Dockerfile"),
            format!("FROM {}\n{steps}", test_inputs::IMAGE),
        )
        .unwrap();
    }
    match environment {
        Environment::Named(name) => (name.to_owned(), None),
        Environment::OfTheTask | Environment::BuiltWith(_) => {
            let image = built(outputs_directory);
            (image.name.clone(), Some(image))
        }
    }
}

fn trialled_as(trial: &Trial<'_>) -> Trialled {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", trial.verifier);
    if trial.unreadable_in_the_harness {
        let unreadable = outputs_directory.harness(0).join("unreadable");
        fs::write(&unreadable, "unread").unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
    }
    let (environment_image, built) = environment_of(&outputs_directory, trial.environment);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: trial.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: trial.key,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let stop = AtomicBool::new(false);
    let environment = sandboxes::Environment {
        image: environment_image.clone(),
        configuration: TaskConfiguration {
            harness_time_limit: trial.harness_time_limit.map(Duration::from_secs),
            verifier_time_limit: Duration::from_secs(30),
            ..TaskConfiguration::default()
        },
        verifier: None,
    };
    let result = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        sandboxes::Trial {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &stop,
        &|_| (),
    );
    Trialled {
        _directory: directory,
        _built: built,
        environment_image,
        outputs_directory,
        result,
    }
}

fn trialled(run_command: &str, verifier: &str) -> Trialled {
    trialled_as(&Trial {
        run_command,
        verifier,
        ..Trial::default()
    })
}

fn within(time_limit: u64, run_command: &str, verifier: &str) -> Trialled {
    trialled_as(&Trial {
        run_command,
        verifier,
        harness_time_limit: Some(time_limit),
        ..Trial::default()
    })
}

fn in_the_environment(environment: Environment<'_>, run_command: &str, verifier: &str) -> Trialled {
    trialled_as(&Trial {
        environment,
        run_command,
        verifier,
        ..Trial::default()
    })
}

fn asked_as(
    environment: Environment<'_>,
    key: Option<&str>,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
) -> Asked {
    Asked {
        trialled: trialled_as(&Trial {
            environment,
            run_command: &test_inputs::one_after_another(of_the_harness),
            verifier: &test_inputs::one_after_another(of_the_verifier),
            key,
            ..Trial::default()
        }),
        of_the_harness,
        of_the_verifier,
    }
}

static LOOKED_AT: LazyLock<Asked> = LazyLock::new(|| {
    asked_as(
        Environment::OfTheTask,
        Some(test_inputs::MODEL_KEY),
        &[
            "cat work.txt",
            "pwd",
            "id -u",
            "echo \"$CLADENC_HARNESS\"; cat /var/opt/cladenc/harness/harness.sh",
            "echo \"$CLADENC_INSTRUCTION\"",
            "env | grep ^CLADENC_MODEL | sort",
            LOOK_FOR_THE_TESTS,
            "ls /",
            "cat",
            "echo $$",
            "hostname",
            FINISH,
        ],
        &[
            "ls -A /logs/verifier",
            "pwd",
            "ls -A /tests",
            "echo \"$0\"",
            "ls /",
            "env | grep -c ^CLADENC_",
            "hostname",
            "echo seen > /logs/verifier/notes.txt",
            FINISHED,
        ],
    )
});

static AS_A_USER_AND_LOOKED_AT: LazyLock<Asked> = LazyLock::new(|| {
    asked_as(
        Environment::BuiltWith(AS_A_USER),
        None,
        &[
            "id -u",
            "stat -c %u /var/opt/cladenc/harness/harness.sh",
            FINISH,
        ],
        &["id -u", FINISHED],
    )
});

static AT_THE_TOP: LazyLock<Asked> = LazyLock::new(|| {
    asked_as(
        Environment::Named(test_inputs::IMAGE),
        None,
        &["pwd", FINISH],
        &[FINISHED],
    )
});

static LEFT_A_DIRECTORY_NO_ONE_CAN_WRITE_IN: LazyLock<(Trialled, io::Result<()>)> =
    LazyLock::new(|| {
        let trialled = trialled(LEAVE_A_DIRECTORY_NO_ONE_CAN_WRITE_IN, FINISHED);
        let removed = fs::remove_dir_all(trialled.outputs_directory.path());
        (trialled, removed)
    });

static IN_AN_ENVIRONMENT_THAT_IS_NOT_ON_THE_MACHINE: LazyLock<Trialled> =
    LazyLock::new(|| in_the_environment(Environment::Named(NOT_ON_THE_MACHINE), FINISH, FINISHED));

fn kept(trialled: &Trialled, file: &str) -> String {
    fs::read_to_string(trialled.outputs_directory.trial(0, TASK).join(file)).unwrap()
}

fn found_by_the_harness(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_harness,
        &kept(&asked.trialled, "stdout"),
    )
}

fn found_by_the_verifier(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_verifier,
        &kept(&asked.trialled, "verifier/stdout"),
    )
}

fn with_its_result(trialled: &Trialled, result: TrialResult) -> bool {
    matches!(&trialled.result, Ok(sandboxes::Trialled { result: found, .. }) if *found == result)
}

fn the_top_of_the_filesystem_of(trialled: &Trialled, added: &[&str]) -> String {
    let mut held: Vec<String> =
        test_inputs::printed_in_a_sandbox_of(&trialled.environment_image, "ls /")
            .lines()
            .chain(added.iter().copied())
            .map(str::to_owned)
            .collect();
    held.sort();
    held.iter().map(|name| format!("{name}\n")).collect()
}

#[test]
fn the_harness_works_on_the_files_the_environment_of_the_task_starts_from() {
    assert_eq!(found_by_the_harness(&LOOKED_AT, "cat work.txt"), "to do\n");
}

#[test]
fn the_harness_works_in_the_working_directory_of_the_environment_of_the_task() {
    assert_eq!(found_by_the_harness(&LOOKED_AT, "pwd"), "/app\n");
}

#[test]
fn the_harness_runs_as_the_user_of_the_environment_of_the_task() {
    assert_eq!(
        found_by_the_harness(&AS_A_USER_AND_LOOKED_AT, "id -u"),
        "1000\n"
    );
}

#[test]
fn the_harness_in_an_environment_that_names_no_user_runs_as_root() {
    assert_eq!(found_by_the_harness(&LOOKED_AT, "id -u"), "0\n");
}

#[test]
fn the_harness_in_an_environment_that_names_no_working_directory_works_at_the_top() {
    assert_eq!(found_by_the_harness(&AT_THE_TOP, "pwd"), "/\n");
}

#[test]
fn no_work_is_kept_where_the_working_directory_of_the_environment_is_the_top() {
    assert!(
        !AT_THE_TOP
            .trialled
            .outputs_directory
            .trial(0, TASK)
            .join("work")
            .exists()
    );
}

#[test]
fn the_harness_is_told_where_its_source_code_is() {
    assert_eq!(
        found_by_the_harness(
            &LOOKED_AT,
            "echo \"$CLADENC_HARNESS\"; cat /var/opt/cladenc/harness/harness.sh"
        ),
        "/var/opt/cladenc/harness\n#!/bin/sh\necho harness\n"
    );
}

#[test]
fn the_source_code_of_the_harness_belongs_to_the_user_of_the_environment() {
    assert_eq!(
        found_by_the_harness(
            &AS_A_USER_AND_LOOKED_AT,
            "stat -c %u /var/opt/cladenc/harness/harness.sh"
        ),
        "1000\n"
    );
}

#[test]
fn the_harness_is_asked_to_do_what_the_instruction_of_the_task_says() {
    assert_eq!(
        found_by_the_harness(&LOOKED_AT, "echo \"$CLADENC_INSTRUCTION\""),
        "Finish the work of t1.\n\n"
    );
}

#[test]
fn the_harness_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    assert_eq!(
        found_by_the_harness(&LOOKED_AT, "env | grep ^CLADENC_MODEL | sort"),
        format!(
            "CLADENC_MODEL={}\nCLADENC_MODEL_KEY={}\nCLADENC_MODEL_URL={}\n",
            test_inputs::MODEL,
            test_inputs::MODEL_KEY,
            test_inputs::MODEL_URL
        )
    );
}

#[test]
fn the_harness_is_given_neither_the_tests_of_the_task_nor_its_own_trials() {
    assert_eq!(found_by_the_harness(&LOOKED_AT, LOOK_FOR_THE_TESTS), "");
}

#[test]
fn the_harness_finds_at_the_top_of_the_filesystem_what_the_environment_holds_with_logs() {
    assert_eq!(
        found_by_the_harness(&LOOKED_AT, "ls /"),
        the_top_of_the_filesystem_of(&LOOKED_AT.trialled, &["logs"])
    );
}

#[test]
fn the_standard_input_of_the_harness_is_empty() {
    assert_eq!(found_by_the_harness(&LOOKED_AT, "cat"), "");
}

#[test]
fn the_standard_error_of_the_harness_is_kept() {
    assert_eq!(kept(&trialled("echo err >&2", FINISHED), "stderr"), "err\n");
}

#[test]
fn every_path_the_harness_added_is_listed_with_the_trial() {
    assert_eq!(
        kept(&trialled("touch /tmp/left", FINISHED), "changes"),
        "C /tmp\nA /tmp/left\n"
    );
}

#[test]
fn a_harness_that_changes_its_source_code_leaves_the_harness_unchanged() {
    let trialled = trialled(
        "echo changed > /var/opt/cladenc/harness/harness.sh",
        FINISHED,
    );
    assert_eq!(
        fs::read_to_string(trialled.outputs_directory.harness(0).join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_that_changes_the_files_of_the_environment_leaves_the_task_unchanged() {
    assert_eq!(
        fs::read_to_string(
            LOOKED_AT
                .trialled
                .outputs_directory
                .task(TASK)
                .join("environment/workspace/work.txt")
        )
        .unwrap(),
        "to do\n"
    );
}

#[test]
fn a_directory_the_harness_left_that_no_one_can_write_in_lets_the_trial_end() {
    assert!(LEFT_A_DIRECTORY_NO_ONE_CAN_WRITE_IN.0.result.is_ok());
}

#[test]
fn a_directory_the_harness_left_that_no_one_can_write_in_lets_the_outputs_be_removed() {
    assert!(LEFT_A_DIRECTORY_NO_ONE_CAN_WRITE_IN.1.is_ok());
}

#[test]
fn a_verdict_of_1_on_what_the_harness_left_gives_a_pass() {
    assert!(with_its_result(&LOOKED_AT.trialled, TrialResult::Pass));
}

#[test]
fn what_the_verifier_leaves_in_logs_verifier_is_kept_with_the_trial() {
    assert_eq!(
        kept(&LOOKED_AT.trialled, "verifier/logs/verifier/notes.txt"),
        "seen\n"
    );
}

#[test]
fn the_verifier_finds_logs_verifier_empty() {
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "ls -A /logs/verifier"),
        ""
    );
}

#[test]
fn a_verdict_the_harness_left_in_logs_verifier_is_not_found_by_the_verifier() {
    assert!(with_its_result(
        &trialled(
            "mkdir -p /logs/verifier && echo 1 > /logs/verifier/reward.txt",
            "true"
        ),
        TrialResult::Inconclusive(Cause::NoVerdict)
    ));
}

#[test]
fn a_verifier_that_removed_logs_verifier_left_no_verdict() {
    assert!(with_its_result(
        &trialled(FINISH, REMOVE_LOGS_VERIFIER),
        TrialResult::Inconclusive(Cause::NoVerdict)
    ));
}

#[test]
fn a_trial_whose_verifier_logs_cannot_be_copied_out_of_its_sandbox_is_given_no_result() {
    assert!(matches!(
        trialled(FINISH, LEAVE_WHAT_CANNOT_BE_COPIED_OUT).result,
        Err(SandboxError::NotCopiedOut(_))
    ));
}

#[test]
fn a_file_the_harness_left_in_tests_is_not_found_by_the_verifier() {
    let trialled = trialled(
        "mkdir -p /tests && echo planted > /tests/planted.sh",
        "ls -A /tests",
    );
    assert_eq!(kept(&trialled, "verifier/stdout"), "test.sh\n");
}

#[test]
fn the_verifier_works_in_the_working_directory_the_harness_worked_in() {
    assert_eq!(found_by_the_verifier(&LOOKED_AT, "pwd"), "/app\n");
}

#[test]
fn the_verifier_runs_as_the_user_the_harness_ran_as() {
    assert_eq!(
        found_by_the_verifier(&AS_A_USER_AND_LOOKED_AT, "id -u"),
        "1000\n"
    );
}

#[test]
fn the_verifier_of_an_environment_with_a_user_can_write_its_verdict() {
    assert!(with_its_result(
        &AS_A_USER_AND_LOOKED_AT.trialled,
        TrialResult::Pass
    ));
}

#[test]
fn the_verifier_finds_the_tests_of_its_task_at_tests() {
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "ls -A /tests"),
        "test.sh\n"
    );
}

#[test]
fn the_verifier_cannot_change_the_tests_of_its_task() {
    let trialled = trialled(FINISH, "echo changed > /tests/test.sh");
    assert_eq!(
        fs::read_to_string(trialled.outputs_directory.task(TASK).join("tests/test.sh")).unwrap(),
        "#!/bin/sh\necho changed > /tests/test.sh\n"
    );
}

#[test]
fn the_verifier_that_is_run_is_the_test_sh_of_the_tests_of_its_task() {
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "echo \"$0\""),
        "/tests/test.sh\n"
    );
}

#[test]
fn the_verifier_finds_at_the_top_of_the_filesystem_what_the_environment_holds_with_logs_and_tests()
{
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "ls /"),
        the_top_of_the_filesystem_of(&LOOKED_AT.trialled, &["logs", "tests"])
    );
}

#[test]
fn the_verifier_is_given_no_variable_of_cladenc() {
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "env | grep -c ^CLADENC_"),
        "0\n"
    );
}

#[test]
fn the_verifier_finds_what_the_harness_left_outside_the_working_directory() {
    assert!(with_its_result(
        &trialled(
            "echo done > /tmp/left",
            "grep -q done /tmp/left && echo 1 > /logs/verifier/reward.txt"
        ),
        TrialResult::Pass
    ));
}

#[test]
fn the_verifier_finds_a_process_that_the_harness_left_running() {
    assert!(with_its_result(
        &trialled(LEAVE_A_PROCESS_RUNNING, THE_PROCESS_STILL_RUNS),
        TrialResult::Pass
    ));
}

#[test]
fn the_verifier_runs_in_the_sandbox_the_harness_ran_in() {
    assert_eq!(
        found_by_the_verifier(&LOOKED_AT, "hostname"),
        found_by_the_harness(&LOOKED_AT, "hostname")
    );
}

#[test]
fn a_trial_that_ended_leaves_no_sandbox_behind() {
    let directory = test_inputs::temporary_directory();
    let id_file = directory.path().join("id");
    fs::write(&id_file, found_by_the_harness(&LOOKED_AT, "hostname")).unwrap();
    assert!(test_inputs::sandbox_is_gone(&id_file));
}

#[test]
fn a_trial_whose_environment_is_not_on_the_machine_is_an_error() {
    assert!(matches!(
        IN_AN_ENVIRONMENT_THAT_IS_NOT_ON_THE_MACHINE.result,
        Err(SandboxError::Docker(_))
    ));
}

#[test]
fn what_docker_said_of_a_trial_whose_environment_is_not_on_the_machine_is_kept() {
    let said = IN_AN_ENVIRONMENT_THAT_IS_NOT_ON_THE_MACHINE
        .outputs_directory
        .running_trial(0, TASK)
        .join("stderr");
    assert!(
        fs::read_to_string(said)
            .unwrap()
            .contains(NOT_ON_THE_MACHINE)
    );
}

#[test]
fn what_a_harness_that_ran_past_its_time_limit_left_is_still_judged() {
    assert!(with_its_result(
        &within(2, FINISH_AND_STAY, FINISHED),
        TrialResult::Pass
    ));
}

#[test]
fn what_a_harness_that_ran_past_its_time_limit_left_running_is_ended_before_the_verifier_runs() {
    assert!(with_its_result(
        &within(
            3,
            LEAVE_A_PROCESS_RUNNING_AND_STAY,
            THE_PROCESS_NO_LONGER_RUNS
        ),
        TrialResult::Pass
    ));
}

#[test]
fn the_command_of_a_harness_is_not_the_first_process_of_its_sandbox() {
    assert_ne!(found_by_the_harness(&LOOKED_AT, "echo $$"), "1\n");
}

#[test]
fn a_trial_whose_harness_cannot_be_copied_into_its_sandbox_is_not_carried_out() {
    let trialled = trialled_as(&Trial {
        unreadable_in_the_harness: true,
        ..Trial::default()
    });
    assert!(matches!(trialled.result, Err(SandboxError::Io(_))));
}
