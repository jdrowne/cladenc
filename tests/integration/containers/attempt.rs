use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, HarnessEnding, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const SAY_THE_SECONDS_OF_THE_TIMER: &str = "p=$PPID; \
    while [ \"$(tr '\\0' '\\n' < /proc/$p/cmdline | wc -l)\" -lt 5 ]; \
    do p=$(awk '/^PPid/ {print $2}' /proc/$p/status); done; \
    tr '\\0' '\\n' < /proc/$p/cmdline | tail -n 1";
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
const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const TASK: &str = "benchmark/t1";
const AS_A_USER: &str = "RUN mkdir /app && chown 1000:1000 /app\nUSER 1000\nWORKDIR /app\n\
                         COPY --chown=1000:1000 workspace .\n";

struct Attempted {
    _directory: TempDir,
    _built: Option<Image>,
    environment_image: String,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

#[derive(Clone, Copy)]
enum Environment<'a> {
    OfTheTask,
    BuiltWith(&'a str),
    Named(&'a str),
}

struct Attempt<'a> {
    environment: Environment<'a>,
    run_command: &'a str,
    grader: &'a str,
    key: Option<&'a str>,
    harness: u64,
    harness_time_limit: Option<u64>,
    grader_time_limit: u64,
    unreadable_in_the_harness: bool,
}

impl Default for Attempt<'_> {
    fn default() -> Self {
        Self {
            environment: Environment::OfTheTask,
            run_command: FINISH,
            grader: FINISHED,
            key: None,
            harness: 0,
            harness_time_limit: Some(30),
            grader_time_limit: 30,
            unreadable_in_the_harness: false,
        }
    }
}

fn built(outputs_directory: &OutputsDirectory) -> Image {
    let image = test_inputs::image();
    containers::build_environment(
        outputs_directory,
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

fn attempted_as(attempt: &Attempt<'_>) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", attempt.grader);
    if attempt.unreadable_in_the_harness {
        let unreadable = outputs_directory.harness(0).join("unreadable");
        fs::write(&unreadable, "unread").unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
    }
    let (environment_image, built) = environment_of(&outputs_directory, attempt.environment);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: attempt.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: attempt.key,
        },
        procreation_timeout: 30,
    };
    let stop = AtomicBool::new(false);
    let environment = containers::Environment {
        image: environment_image.clone(),
        configuration: TaskConfiguration {
            harness_time_limit: attempt.harness_time_limit.map(Duration::from_secs),
            grader_time_limit: Duration::from_secs(attempt.grader_time_limit),
            ..TaskConfiguration::default()
        },
    };
    let result = containers::run_attempt(
        &outputs_directory,
        &launch,
        containers::Attempt {
            harness: attempt.harness,
            task: TASK,
            environment: &environment,
        },
        &stop,
    );
    Attempted {
        _directory: directory,
        _built: built,
        environment_image,
        outputs_directory,
        result,
    }
}

fn attempted(run_command: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        run_command,
        grader,
        ..Attempt::default()
    })
}

fn within(time_limit: u64, run_command: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        run_command,
        grader,
        harness_time_limit: Some(time_limit),
        ..Attempt::default()
    })
}

fn graded_within(time_limit: u64, run_command: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        run_command,
        grader,
        grader_time_limit: time_limit,
        ..Attempt::default()
    })
}

fn ran_out_of_time(attempted: &Attempted) -> Option<bool> {
    attempted
        .result
        .as_ref()
        .ok()
        .map(|attempted| attempted.ending == HarnessEnding::Stopped)
}

fn in_the_environment(environment: Environment<'_>, run_command: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        environment,
        run_command,
        grader,
        ..Attempt::default()
    })
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
}

fn left_by_the_harness(run_command: &str, file: &str) -> String {
    kept(&attempted(run_command, FINISHED), &format!("work/{file}"))
}

fn printed_by_the_grader(grader: &str) -> String {
    kept(&attempted(FINISH, grader), "grader/stdout")
}

fn told_of_the_model(key: Option<&str>) -> String {
    let attempted = attempted_as(&Attempt {
        run_command: "env | grep ^CLADENC_MODEL | sort > model",
        key,
        ..Attempt::default()
    });
    kept(&attempted, "work/model")
}

fn with_its_result(attempted: &Attempted, result: AttemptResult) -> bool {
    matches!(&attempted.result, Ok(containers::Attempted { result: found, .. }) if *found == result)
}

fn the_top_of_the_filesystem_of(attempted: &Attempted, added: &[&str]) -> String {
    let mut held: Vec<String> =
        test_inputs::printed_in_a_container_of(&attempted.environment_image, "ls /")
            .lines()
            .chain(added.iter().copied())
            .map(str::to_owned)
            .collect();
    held.sort();
    held.iter().map(|name| format!("{name}\n")).collect()
}

#[test]
fn the_harness_works_on_the_files_the_environment_of_the_task_starts_from() {
    assert_eq!(
        left_by_the_harness("cat work.txt > read", "read"),
        "to do\n"
    );
}

#[test]
fn the_harness_works_in_the_working_directory_of_the_environment_of_the_task() {
    assert_eq!(left_by_the_harness("pwd > where", "where"), "/app\n");
}

#[test]
fn the_harness_runs_as_the_user_of_the_environment_of_the_task() {
    let attempted = in_the_environment(Environment::BuiltWith(AS_A_USER), "id -u", FINISHED);
    assert_eq!(kept(&attempted, "stdout"), "1000\n");
}

#[test]
fn the_harness_in_an_environment_that_names_no_user_runs_as_root() {
    assert_eq!(left_by_the_harness("id -u > who", "who"), "0\n");
}

#[test]
fn the_harness_in_an_environment_that_names_no_working_directory_works_at_the_top() {
    let attempted = in_the_environment(Environment::Named(test_inputs::IMAGE), "pwd", FINISHED);
    assert_eq!(kept(&attempted, "stdout"), "/\n");
}

#[test]
fn no_work_is_kept_where_the_working_directory_of_the_environment_is_the_top() {
    let attempted = in_the_environment(Environment::Named(test_inputs::IMAGE), FINISH, FINISHED);
    assert!(
        !attempted
            .outputs_directory
            .attempt(0, TASK)
            .join("work")
            .exists()
    );
}

#[test]
fn the_harness_is_told_where_its_source_code_is() {
    assert_eq!(
        left_by_the_harness(
            "echo \"$CLADENC_HARNESS\" > told; cat /var/lib/cladenc/harness/harness.sh >> told",
            "told"
        ),
        "/var/lib/cladenc/harness\n#!/bin/sh\necho harness\n"
    );
}

#[test]
fn the_source_code_of_the_harness_belongs_to_the_user_of_the_environment() {
    let attempted = in_the_environment(
        Environment::BuiltWith(AS_A_USER),
        "stat -c %u /var/lib/cladenc/harness/harness.sh",
        FINISHED,
    );
    assert_eq!(kept(&attempted, "stdout"), "1000\n");
}

#[test]
fn the_harness_is_asked_to_do_what_the_specification_of_the_task_says() {
    assert_eq!(
        left_by_the_harness("echo \"$CLADENC_INSTRUCTION\" > asked", "asked"),
        "Finish the work of t1.\n\n"
    );
}

#[test]
fn the_harness_is_told_the_model_the_address_it_is_served_at_and_its_key() {
    assert_eq!(
        told_of_the_model(Some(test_inputs::MODEL_KEY)),
        format!(
            "CLADENC_MODEL={}\nCLADENC_MODEL_KEY={}\nCLADENC_MODEL_URL={}\n",
            test_inputs::MODEL,
            test_inputs::MODEL_KEY,
            test_inputs::MODEL_URL
        )
    );
}

#[test]
fn the_harness_of_a_directed_evolution_with_no_key_is_given_none() {
    assert_eq!(
        told_of_the_model(None),
        format!(
            "CLADENC_MODEL={}\nCLADENC_MODEL_URL={}\n",
            test_inputs::MODEL,
            test_inputs::MODEL_URL
        )
    );
}

#[test]
fn the_harness_is_given_neither_the_tests_of_the_task_nor_its_own_attempts() {
    assert_eq!(
        left_by_the_harness(
            "ls -d /tests /logs /var/log/cladenc/attempts > given 2> /dev/null",
            "given"
        ),
        ""
    );
}

#[test]
fn the_harness_finds_nothing_at_the_top_of_the_filesystem_that_the_environment_does_not_hold() {
    let attempted = attempted("ls / > top", FINISHED);
    assert_eq!(
        kept(&attempted, "work/top"),
        the_top_of_the_filesystem_of(&attempted, &[])
    );
}

#[test]
fn the_standard_input_of_the_harness_is_empty() {
    assert_eq!(left_by_the_harness("cat > input", "input"), "");
}

#[test]
fn the_standard_output_of_the_harness_is_kept() {
    assert_eq!(kept(&attempted("echo out", FINISHED), "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_the_harness_is_kept() {
    assert_eq!(
        kept(&attempted("echo err >&2", FINISHED), "stderr"),
        "err\n"
    );
}

#[test]
fn the_specification_of_the_task_is_kept_with_the_attempt() {
    assert_eq!(
        kept(&attempted(FINISH, FINISHED), "specification"),
        "Finish the work of t1.\n"
    );
}

#[test]
fn every_path_the_harness_added_is_listed_with_the_attempt() {
    assert_eq!(
        kept(&attempted("touch /tmp/left", FINISHED), "changes"),
        "C /tmp\nA /tmp/left\n"
    );
}

#[test]
fn a_harness_that_changes_its_source_code_leaves_the_harness_unchanged() {
    let attempted = attempted(
        "echo changed > /var/lib/cladenc/harness/harness.sh",
        FINISHED,
    );
    assert_eq!(
        fs::read_to_string(attempted.outputs_directory.harness(0).join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_that_changes_the_files_of_the_environment_leaves_the_task_unchanged() {
    let attempted = attempted(FINISH, FINISHED);
    assert_eq!(
        fs::read_to_string(
            attempted
                .outputs_directory
                .task(TASK)
                .join("environment/workspace/work.txt")
        )
        .unwrap(),
        "to do\n"
    );
}

#[test]
fn a_directory_the_harness_left_that_no_one_can_write_in_lets_the_attempt_end() {
    assert!(
        attempted(LEAVE_A_DIRECTORY_NO_ONE_CAN_WRITE_IN, FINISHED)
            .result
            .is_ok()
    );
}

#[test]
fn a_directory_the_harness_left_that_no_one_can_write_in_lets_the_outputs_be_removed() {
    let attempted = attempted(LEAVE_A_DIRECTORY_NO_ONE_CAN_WRITE_IN, FINISHED);
    assert!(fs::remove_dir_all(attempted.outputs_directory.path()).is_ok());
}

#[test]
fn a_grader_that_scores_1_on_what_the_harness_left_gives_completed() {
    assert!(with_its_result(
        &attempted(FINISH, FINISHED),
        AttemptResult::Completed
    ));
}

#[test]
fn a_grader_that_leaves_no_score_gives_not_completed() {
    assert!(with_its_result(
        &attempted("true", FINISHED),
        AttemptResult::NotCompleted
    ));
}

#[test]
fn a_grader_that_scores_0_gives_not_completed() {
    assert!(with_its_result(
        &attempted(FINISH, "echo 0 > /logs/verifier/reward.txt"),
        AttemptResult::NotCompleted
    ));
}

#[test]
fn a_grader_that_scores_1_and_exits_with_another_status_than_0_gives_completed() {
    assert!(with_its_result(
        &attempted(FINISH, "echo 1 > /logs/verifier/reward.txt; exit 2"),
        AttemptResult::Completed
    ));
}

#[test]
fn a_grader_that_scores_1_on_every_label_of_reward_json_gives_completed() {
    assert!(with_its_result(
        &attempted(
            FINISH,
            "echo '{\"tests\": 1, \"style\": 1.0}' > /logs/verifier/reward.json"
        ),
        AttemptResult::Completed
    ));
}

#[test]
fn a_grader_whose_reward_json_says_less_than_1_on_a_label_gives_not_completed() {
    assert!(with_its_result(
        &attempted(
            FINISH,
            "echo 1 > /logs/verifier/reward.txt; \
             echo '{\"tests\": 1, \"style\": 0.5}' > /logs/verifier/reward.json"
        ),
        AttemptResult::NotCompleted
    ));
}

#[test]
fn the_score_of_the_grader_is_kept_with_the_attempt() {
    assert_eq!(
        kept(
            &attempted(FINISH, FINISHED),
            "grader/logs/verifier/reward.txt"
        ),
        "1\n"
    );
}

#[test]
fn what_the_grader_leaves_in_logs_verifier_is_kept_with_the_attempt() {
    assert_eq!(
        kept(
            &attempted(FINISH, "echo seen > /logs/verifier/notes.txt"),
            "grader/logs/verifier/notes.txt"
        ),
        "seen\n"
    );
}

#[test]
fn the_grader_finds_logs_verifier_empty() {
    assert_eq!(printed_by_the_grader("ls -A /logs/verifier"), "");
}

#[test]
fn a_score_the_harness_left_in_logs_verifier_is_not_found_by_the_grader() {
    assert!(with_its_result(
        &attempted(
            "mkdir -p /logs/verifier && echo 1 > /logs/verifier/reward.txt",
            "true"
        ),
        AttemptResult::NotCompleted
    ));
}

#[test]
fn a_file_the_harness_left_in_tests_is_not_found_by_the_grader() {
    let attempted = attempted(
        "mkdir -p /tests && echo planted > /tests/planted.sh",
        "ls -A /tests",
    );
    assert_eq!(kept(&attempted, "grader/stdout"), "test.sh\n");
}

#[test]
fn a_harness_that_exits_with_another_status_than_0_is_still_checked() {
    assert!(with_its_result(
        &attempted("echo done > work.txt; exit 3", FINISHED),
        AttemptResult::Completed
    ));
}

#[test]
fn the_grader_works_in_the_working_directory_the_harness_worked_in() {
    assert_eq!(printed_by_the_grader("pwd"), "/app\n");
}

#[test]
fn the_grader_runs_as_the_user_the_harness_ran_as() {
    let attempted = in_the_environment(Environment::BuiltWith(AS_A_USER), FINISH, "id -u");
    assert_eq!(kept(&attempted, "grader/stdout"), "1000\n");
}

#[test]
fn the_grader_of_an_environment_with_a_user_can_score() {
    assert!(with_its_result(
        &in_the_environment(Environment::BuiltWith(AS_A_USER), FINISH, FINISHED),
        AttemptResult::Completed
    ));
}

#[test]
fn the_grader_finds_the_tests_of_its_task_at_tests() {
    assert_eq!(printed_by_the_grader("ls -A /tests"), "test.sh\n");
}

#[test]
fn the_grader_cannot_change_the_tests_of_its_task() {
    let attempted = attempted(FINISH, "echo changed > /tests/test.sh");
    assert_eq!(
        fs::read_to_string(attempted.outputs_directory.task(TASK).join("tests/test.sh")).unwrap(),
        "#!/bin/sh\necho changed > /tests/test.sh\n"
    );
}

#[test]
fn the_grader_that_is_run_is_the_test_sh_of_the_tests_of_its_task() {
    assert_eq!(printed_by_the_grader("echo \"$0\""), "/tests/test.sh\n");
}

#[test]
fn the_grader_finds_at_the_top_of_the_filesystem_what_the_environment_holds_with_logs_and_tests() {
    let attempted = attempted(FINISH, "ls /");
    assert_eq!(
        kept(&attempted, "grader/stdout"),
        the_top_of_the_filesystem_of(&attempted, &["logs", "tests"])
    );
}

#[test]
fn the_grader_is_given_no_variable_of_cladenc() {
    assert_eq!(printed_by_the_grader("env | grep -c ^CLADENC_"), "0\n");
}

#[test]
fn the_grader_finds_what_the_harness_left_outside_the_working_directory() {
    assert!(with_its_result(
        &attempted(
            "echo done > /tmp/left",
            "grep -q done /tmp/left && echo 1 > /logs/verifier/reward.txt"
        ),
        AttemptResult::Completed
    ));
}

#[test]
fn the_grader_finds_a_process_that_the_harness_left_running() {
    assert!(with_its_result(
        &attempted(LEAVE_A_PROCESS_RUNNING, THE_PROCESS_STILL_RUNS),
        AttemptResult::Completed
    ));
}

#[test]
fn the_grader_runs_in_the_container_the_harness_ran_in() {
    let attempted = attempted("hostname", "hostname");
    assert_eq!(
        kept(&attempted, "grader/stdout"),
        kept(&attempted, "stdout")
    );
}

#[test]
fn an_attempt_that_ended_leaves_no_container_behind() {
    let attempted = attempted("hostname", FINISHED);
    assert!(test_inputs::container_is_gone(
        &attempted.outputs_directory.attempt(0, TASK).join("stdout")
    ));
}

#[test]
fn an_attempt_whose_environment_is_not_on_the_machine_is_an_error() {
    let attempted = in_the_environment(Environment::Named(NOT_ON_THE_MACHINE), FINISH, FINISHED);
    assert!(matches!(attempted.result, Err(ContainerError::Docker(_))));
}

#[test]
fn what_docker_said_of_an_attempt_whose_environment_is_not_on_the_machine_is_kept() {
    let attempted = in_the_environment(Environment::Named(NOT_ON_THE_MACHINE), FINISH, FINISHED);
    let said = attempted
        .outputs_directory
        .running_attempt(0, TASK)
        .join("stderr");
    assert!(
        fs::read_to_string(said)
            .unwrap()
            .contains(NOT_ON_THE_MACHINE)
    );
}

#[test]
fn the_standard_output_of_the_grader_is_kept_apart_from_that_of_the_harness() {
    assert_eq!(printed_by_the_grader("echo out"), "out\n");
}

#[test]
fn the_standard_error_of_the_grader_is_kept_apart_from_that_of_the_harness() {
    assert_eq!(
        kept(&attempted(FINISH, "echo err >&2"), "grader/stderr"),
        "err\n"
    );
}

#[test]
fn an_attempt_that_ended_leaves_its_result_in_its_place() {
    assert_eq!(kept(&attempted(FINISH, FINISHED), "result"), "completed\n");
}

#[test]
fn an_attempt_that_ended_keeps_no_copy_of_the_harness() {
    let attempted = attempted(FINISH, FINISHED);
    assert!(
        !attempted
            .outputs_directory
            .attempt(0, TASK)
            .join("harness")
            .exists()
    );
}

#[test]
fn what_a_harness_that_ran_past_its_time_limit_left_is_still_judged() {
    assert!(with_its_result(
        &within(2, FINISH_AND_STAY, FINISHED),
        AttemptResult::Completed
    ));
}

#[test]
fn a_harness_that_ran_past_its_time_limit_is_checked() {
    let attempted = within(2, "sleep 300", "echo checked");
    assert_eq!(kept(&attempted, "grader/stdout"), "checked\n");
}

#[test]
fn a_harness_that_ran_past_its_time_limit_comes_back_as_having_run_out_of_time() {
    assert_eq!(
        ran_out_of_time(&within(2, "sleep 300", FINISHED)),
        Some(true)
    );
}

#[test]
fn a_harness_that_ended_by_itself_comes_back_as_not_having_run_out_of_time() {
    assert_eq!(ran_out_of_time(&attempted(FINISH, FINISHED)), Some(false));
}

#[test]
fn an_attempt_whose_harness_ran_past_its_time_limit_keeps_the_word_stopped() {
    assert_eq!(
        kept(&within(2, "sleep 300", FINISHED), "ending"),
        "stopped\n"
    );
}

#[test]
fn an_attempt_whose_harness_ended_by_itself_keeps_the_word_ended() {
    assert_eq!(kept(&attempted(FINISH, FINISHED), "ending"), "ended\n");
}

#[test]
fn what_a_harness_that_ran_past_its_time_limit_left_running_is_ended_before_the_grader_runs() {
    assert!(with_its_result(
        &within(
            3,
            LEAVE_A_PROCESS_RUNNING_AND_STAY,
            THE_PROCESS_NO_LONGER_RUNS
        ),
        AttemptResult::Completed
    ));
}

#[test]
fn a_harness_whose_task_gives_it_no_time_limit_is_given_none() {
    let attempted = attempted_as(&Attempt {
        run_command: SAY_THE_SECONDS_OF_THE_TIMER,
        harness_time_limit: None,
        ..Attempt::default()
    });
    assert!(kept(&attempted, "stdout").trim().parse::<u64>().unwrap() >= u64::from(u32::MAX));
}

#[test]
fn a_grader_that_runs_past_its_time_limit_times_out() {
    assert!(with_its_result(
        &graded_within(2, FINISH, "sleep 300"),
        AttemptResult::TimedOut
    ));
}

#[test]
fn the_timer_of_a_harness_waits_for_the_time_limit_of_the_harness_and_five_seconds_more() {
    let seconds = kept(&attempted(SAY_THE_SECONDS_OF_THE_TIMER, FINISHED), "stdout");
    assert!((30..=35).contains(&seconds.trim().parse::<u64>().unwrap()));
}

#[test]
fn the_time_of_a_grader_is_counted_from_the_moment_the_harness_ends() {
    let seconds = kept(
        &graded_within(40, "sleep 8", SAY_THE_SECONDS_OF_THE_TIMER),
        "grader/stdout",
    );
    assert!((40..=45).contains(&seconds.trim().parse::<u64>().unwrap()));
}

#[test]
fn the_command_of_a_harness_is_not_the_first_process_of_its_container() {
    assert_ne!(left_by_the_harness("echo $$ > process", "process"), "1\n");
}

#[test]
fn a_harness_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        attempted("exit 125", FINISHED).result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn a_grader_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        attempted(FINISH, "exit 125").result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn what_docker_said_of_a_grader_it_did_not_start_is_kept_with_the_grader() {
    let attempted = attempted(FINISH, "exit 125");
    let kept = attempted
        .outputs_directory
        .running_attempt(0, TASK)
        .join("grader/stderr");
    assert!(matches!(attempted.result, Err(ContainerError::Docker(said)) if said == kept));
}

#[test]
fn after_an_error_the_directory_of_an_attempt_is_still_under_running() {
    let attempted = attempted("exit 125", FINISHED);
    assert!(
        attempted
            .outputs_directory
            .running_attempt(0, TASK)
            .join("stdout")
            .is_file()
    );
}

#[test]
fn an_attempt_of_a_harness_that_is_not_there_is_an_error() {
    let attempted = attempted_as(&Attempt {
        harness: 9,
        ..Attempt::default()
    });
    assert!(matches!(attempted.result, Err(ContainerError::Io(_))));
}

#[test]
fn an_attempt_whose_harness_cannot_be_copied_into_its_container_is_not_carried_out() {
    let attempted = attempted_as(&Attempt {
        unreadable_in_the_harness: true,
        ..Attempt::default()
    });
    assert!(matches!(attempted.result, Err(ContainerError::Io(_))));
}
