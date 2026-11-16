use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::Score;
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt";
const SAY_THE_SECONDS_OF_THE_TIMER: &str = "tr '\\0' '\\n' < /proc/1/cmdline | tail -n 1 > seconds";

struct Assessed {
    _directory: TempDir,
    workspace: Workspace,
    result: Result<Score, ContainerError>,
}

struct Assessment<'a> {
    run_command: &'a str,
    check: &'a str,
    key: Option<&'a str>,
    harness: u64,
    time_limit: u64,
}

fn assessed_as(assessment: &Assessment<'_>) -> Assessed {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let run_directory = directory.path().join("run");
    let workspace = Workspace::create(&run_directory, &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_check(&run_directory, "t1", assessment.check);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: assessment.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: assessment.key,
        },
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_assessment(
        &workspace,
        &launch,
        assessment.harness,
        "benchmark/t1",
        assessment.time_limit,
        &stop,
    );
    Assessed {
        _directory: directory,
        workspace,
        result,
    }
}

fn assessed(run_command: &str, check: &str) -> Assessed {
    assessed_as(&Assessment {
        run_command,
        check,
        key: None,
        harness: 0,
        time_limit: 30,
    })
}

fn within(time_limit: u64, run_command: &str, check: &str) -> Assessed {
    assessed_as(&Assessment {
        run_command,
        check,
        key: None,
        harness: 0,
        time_limit,
    })
}

fn kept(assessed: &Assessed, file: &str) -> String {
    fs::read_to_string(assessed.workspace.assessment(0, "benchmark/t1").join(file)).unwrap()
}

fn left_by_the_harness(run_command: &str, file: &str) -> String {
    kept(&assessed(run_command, FINISHED), &format!("work/{file}"))
}

fn left_by_the_check(check: &str, file: &str) -> String {
    kept(&assessed(FINISH, check), &format!("work/{file}"))
}

fn told_of_the_model(key: Option<&str>) -> String {
    let assessed = assessed_as(&Assessment {
        run_command: "env | grep ^CLADENC_MODEL | sort > model",
        check: FINISHED,
        key,
        harness: 0,
        time_limit: 30,
    });
    kept(&assessed, "work/model")
}

#[test]
fn the_harness_works_on_a_copy_of_the_files_of_the_task() {
    assert_eq!(
        left_by_the_harness("cat work.txt > read", "read"),
        "to do\n"
    );
}

#[test]
fn the_working_directory_of_the_harness_holds_the_files_of_the_task() {
    assert_eq!(
        left_by_the_harness("pwd > where", "where"),
        "/var/lib/cladenc/work\n"
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
fn the_harness_is_asked_to_do_what_the_statement_of_the_task_says() {
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
fn the_harness_is_given_neither_the_task_nor_its_own_assessments() {
    assert_eq!(
        left_by_the_harness(
            "ls -d /var/lib/cladenc/task /var/log/cladenc/assessments > given 2> /dev/null",
            "given"
        ),
        ""
    );
}

#[test]
fn the_harness_finds_nothing_at_the_top_of_the_filesystem_that_its_image_does_not_hold() {
    assert_eq!(
        left_by_the_harness("ls / > top", "top"),
        test_inputs::printed_in_a_container_of(test_inputs::IMAGE, "ls /")
    );
}

#[test]
fn the_standard_input_of_the_harness_is_empty() {
    assert_eq!(left_by_the_harness("cat > input", "input"), "");
}

#[test]
fn the_standard_output_of_the_harness_is_kept() {
    assert_eq!(kept(&assessed("echo out", FINISHED), "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_the_harness_is_kept() {
    assert_eq!(kept(&assessed("echo err >&2", FINISHED), "stderr"), "err\n");
}

#[test]
fn the_statement_of_the_task_is_kept_with_the_assessment() {
    assert_eq!(
        kept(&assessed(FINISH, FINISHED), "statement"),
        "Finish the work of t1.\n"
    );
}

#[test]
fn a_harness_that_changes_its_source_code_leaves_the_harness_unchanged() {
    let assessed = assessed(
        "echo changed > /var/lib/cladenc/harness/harness.sh",
        FINISHED,
    );
    assert_eq!(
        fs::read_to_string(assessed.workspace.harness(0).join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_that_changes_the_files_leaves_the_task_unchanged() {
    let assessed = assessed(FINISH, FINISHED);
    assert_eq!(
        fs::read_to_string(
            assessed
                .workspace
                .task("benchmark/t1")
                .join("files/work.txt")
        )
        .unwrap(),
        "to do\n"
    );
}

#[test]
fn a_check_that_exits_with_0_on_what_the_harness_left_scores_a_success() {
    assert!(matches!(
        assessed(FINISH, FINISHED).result,
        Ok(Score::Success)
    ));
}

#[test]
fn a_check_that_exits_with_1_scores_a_failure() {
    assert!(matches!(
        assessed("true", FINISHED).result,
        Ok(Score::Failure)
    ));
}

#[test]
fn a_check_that_exits_with_another_status_scores_a_failure() {
    assert!(matches!(
        assessed(FINISH, "exit 2").result,
        Ok(Score::Failure)
    ));
}

#[test]
fn a_harness_that_exits_with_another_status_than_0_is_still_checked() {
    assert!(matches!(
        assessed("echo done > work.txt; exit 3", FINISHED).result,
        Ok(Score::Success)
    ));
}

#[test]
fn the_working_directory_of_the_check_holds_what_the_harness_left() {
    assert_eq!(
        left_by_the_check("pwd > where", "where"),
        "/var/lib/cladenc/work\n"
    );
}

#[test]
fn the_check_is_told_where_the_directory_of_its_task_is() {
    assert_eq!(
        left_by_the_check(
            "echo \"$CLADENC_TASK\" > told; cat /var/lib/cladenc/task/statement >> told",
            "told"
        ),
        "/var/lib/cladenc/task\nFinish the work of t1.\n"
    );
}

#[test]
fn the_check_is_given_the_files_the_work_started_from() {
    assert_eq!(
        left_by_the_check(
            "cat /var/lib/cladenc/task/files/work.txt > started",
            "started"
        ),
        "to do\n"
    );
}

#[test]
fn the_check_cannot_change_its_task() {
    let assessed = assessed(FINISH, "echo changed > /var/lib/cladenc/task/statement");
    assert_eq!(
        fs::read_to_string(assessed.workspace.task("benchmark/t1").join("statement")).unwrap(),
        "Finish the work of t1.\n"
    );
}

#[test]
fn the_check_that_is_run_is_the_one_in_the_directory_of_its_task() {
    assert_eq!(
        left_by_the_check("echo \"$0\" > called", "called"),
        "/var/lib/cladenc/task/check\n"
    );
}

#[test]
fn the_check_finds_nothing_at_the_top_of_the_filesystem_that_its_image_does_not_hold() {
    assert_eq!(
        left_by_the_check("ls / > top", "top"),
        test_inputs::printed_in_a_container_of(test_inputs::IMAGE, "ls /")
    );
}

#[test]
fn the_check_is_told_nothing_of_the_model() {
    assert_eq!(
        left_by_the_check("env | grep -c ^CLADENC_MODEL > told", "told"),
        "0\n"
    );
}

#[test]
fn the_standard_output_of_the_check_is_kept_apart_from_that_of_the_harness() {
    assert_eq!(kept(&assessed(FINISH, "echo out"), "check/stdout"), "out\n");
}

#[test]
fn the_standard_error_of_the_check_is_kept_apart_from_that_of_the_harness() {
    assert_eq!(
        kept(&assessed(FINISH, "echo err >&2"), "check/stderr"),
        "err\n"
    );
}

#[test]
fn an_assessment_that_ended_leaves_the_score_of_its_harness_in_its_place() {
    assert_eq!(kept(&assessed(FINISH, FINISHED), "score"), "success\n");
}

#[test]
fn an_assessment_that_ended_keeps_no_copy_of_the_harness() {
    let assessed = assessed(FINISH, FINISHED);
    assert!(
        !assessed
            .workspace
            .assessment(0, "benchmark/t1")
            .join("harness")
            .exists()
    );
}

#[test]
fn a_harness_that_runs_past_the_time_limit_times_out() {
    assert!(matches!(
        within(1, "sleep 300", FINISHED).result,
        Ok(Score::TimedOut)
    ));
}

#[test]
fn a_harness_that_timed_out_is_not_checked() {
    let assessed = within(1, "sleep 300", "touch checked");
    assert!(
        !assessed
            .workspace
            .assessment(0, "benchmark/t1")
            .join("work/checked")
            .exists()
    );
}

#[test]
fn a_check_that_runs_past_what_is_left_of_the_time_limit_times_out() {
    assert!(matches!(
        within(20, FINISH, "sleep 300").result,
        Ok(Score::TimedOut)
    ));
}

#[test]
fn the_timer_of_a_harness_waits_five_seconds_longer_than_the_time_limit() {
    assert_eq!(
        left_by_the_harness(SAY_THE_SECONDS_OF_THE_TIMER, "seconds"),
        "35\n"
    );
}

#[test]
fn the_timer_of_a_check_waits_for_what_the_harness_left_of_the_time_limit() {
    let seconds = kept(
        &assessed("sleep 2", SAY_THE_SECONDS_OF_THE_TIMER),
        "work/seconds",
    );
    assert!(seconds.trim().parse::<u64>().unwrap() <= 33);
}

#[test]
fn the_command_of_a_harness_is_not_the_first_process_of_its_container() {
    assert_ne!(left_by_the_harness("echo $$ > process", "process"), "1\n");
}

#[test]
fn a_harness_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        assessed("exit 125", FINISHED).result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn a_check_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        assessed(FINISH, "exit 125").result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn what_docker_said_of_a_check_it_did_not_start_is_kept_with_the_check() {
    let assessed = assessed(FINISH, "exit 125");
    let kept = assessed
        .workspace
        .running_assessment(0, "benchmark/t1")
        .join("check/stderr");
    assert!(matches!(assessed.result, Err(ContainerError::Docker(said)) if said == kept));
}

#[test]
fn after_an_error_the_directory_of_an_assessment_is_still_under_running() {
    let assessed = assessed("exit 125", FINISHED);
    assert!(
        assessed
            .workspace
            .running_assessment(0, "benchmark/t1")
            .join("stdout")
            .is_file()
    );
}

#[test]
fn an_assessment_of_a_harness_that_is_not_there_is_an_error() {
    let assessed = assessed_as(&Assessment {
        run_command: FINISH,
        check: FINISHED,
        key: None,
        harness: 9,
        time_limit: 30,
    });
    assert!(matches!(assessed.result, Err(ContainerError::Io(_))));
}
