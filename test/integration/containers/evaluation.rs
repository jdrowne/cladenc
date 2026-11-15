use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::EvaluationOutcome;
use cladenc::model::Model;
use cladenc::workspace::Workspace;
use tempfile::TempDir;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt";

struct Evaluated {
    _directory: TempDir,
    workspace: Workspace,
    result: Result<EvaluationOutcome, ContainerError>,
}

struct Evaluation<'a> {
    run_command: &'a str,
    check: &'a str,
    key: Option<&'a str>,
    harness: u64,
    time_limit: u64,
}

fn evaluated_as(evaluation: &Evaluation<'_>) -> Evaluated {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let run_directory = directory.path().join("run");
    let workspace = Workspace::create(&run_directory, &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_check(&run_directory, "t1", evaluation.check);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: evaluation.run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: evaluation.key,
        },
    };
    let stop = AtomicBool::new(false);
    let result = containers::run_evaluation(
        &workspace,
        &launch,
        evaluation.harness,
        "benchmark/t1",
        evaluation.time_limit,
        &stop,
    );
    Evaluated {
        _directory: directory,
        workspace,
        result,
    }
}

fn evaluated(run_command: &str, check: &str) -> Evaluated {
    evaluated_as(&Evaluation {
        run_command,
        check,
        key: None,
        harness: 0,
        time_limit: 30,
    })
}

fn within(time_limit: u64, run_command: &str, check: &str) -> Evaluated {
    evaluated_as(&Evaluation {
        run_command,
        check,
        key: None,
        harness: 0,
        time_limit,
    })
}

fn kept(evaluated: &Evaluated, file: &str) -> String {
    fs::read_to_string(evaluated.workspace.evaluation(0, "benchmark/t1").join(file)).unwrap()
}

fn left_by_the_harness(run_command: &str, file: &str) -> String {
    kept(&evaluated(run_command, FINISHED), &format!("work/{file}"))
}

fn left_by_the_check(check: &str, file: &str) -> String {
    kept(&evaluated(FINISH, check), &format!("work/{file}"))
}

fn told_of_the_model(key: Option<&str>) -> String {
    let evaluated = evaluated_as(&Evaluation {
        run_command: "env | grep ^CLADENC_MODEL | sort > model",
        check: FINISHED,
        key,
        harness: 0,
        time_limit: 30,
    });
    kept(&evaluated, "work/model")
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
    assert_eq!(left_by_the_harness("pwd > where", "where"), "/work\n");
}

#[test]
fn the_harness_is_told_where_its_source_code_is() {
    assert_eq!(
        left_by_the_harness(
            "echo \"$CLADENC_HARNESS\" > told; cat /harness/harness.sh >> told",
            "told"
        ),
        "/harness\n#!/bin/sh\necho harness\n"
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
fn the_harness_of_a_search_with_no_key_is_given_none() {
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
fn the_harness_is_given_neither_the_task_nor_its_own_evaluations() {
    assert_eq!(
        left_by_the_harness("ls -d /task /evaluations > given 2> /dev/null", "given"),
        ""
    );
}

#[test]
fn the_standard_input_of_the_harness_is_empty() {
    assert_eq!(left_by_the_harness("cat > input", "input"), "");
}

#[test]
fn the_standard_output_of_the_harness_is_kept() {
    assert_eq!(kept(&evaluated("echo out", FINISHED), "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_the_harness_is_kept() {
    assert_eq!(
        kept(&evaluated("echo err >&2", FINISHED), "stderr"),
        "err\n"
    );
}

#[test]
fn the_statement_of_the_task_is_kept_with_the_evaluation() {
    assert_eq!(
        kept(&evaluated(FINISH, FINISHED), "statement"),
        "Finish the work of t1.\n"
    );
}

#[test]
fn a_harness_that_changes_its_source_code_leaves_the_harness_unchanged() {
    let evaluated = evaluated("echo changed > /harness/harness.sh", FINISHED);
    assert_eq!(
        fs::read_to_string(evaluated.workspace.harness(0).join("harness.sh")).unwrap(),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_that_changes_the_files_leaves_the_task_unchanged() {
    let evaluated = evaluated(FINISH, FINISHED);
    assert_eq!(
        fs::read_to_string(
            evaluated
                .workspace
                .task("benchmark/t1")
                .join("files/work.txt")
        )
        .unwrap(),
        "to do\n"
    );
}

#[test]
fn a_check_that_exits_with_0_on_what_the_harness_left_is_a_success() {
    assert!(matches!(
        evaluated(FINISH, FINISHED).result,
        Ok(EvaluationOutcome::Success)
    ));
}

#[test]
fn a_check_that_exits_with_1_is_a_failure() {
    assert!(matches!(
        evaluated("true", FINISHED).result,
        Ok(EvaluationOutcome::Failure)
    ));
}

#[test]
fn a_check_that_exits_with_another_status_is_a_failure() {
    assert!(matches!(
        evaluated(FINISH, "exit 2").result,
        Ok(EvaluationOutcome::Failure)
    ));
}

#[test]
fn a_harness_that_exits_with_another_status_than_0_is_still_checked() {
    assert!(matches!(
        evaluated("echo done > work.txt; exit 3", FINISHED).result,
        Ok(EvaluationOutcome::Success)
    ));
}

#[test]
fn the_working_directory_of_the_check_holds_what_the_harness_left() {
    assert_eq!(left_by_the_check("pwd > where", "where"), "/work\n");
}

#[test]
fn the_check_is_told_where_the_directory_of_its_task_is() {
    assert_eq!(
        left_by_the_check(
            "echo \"$CLADENC_TASK\" > told; cat /task/statement >> told",
            "told"
        ),
        "/task\nFinish the work of t1.\n"
    );
}

#[test]
fn the_check_is_given_the_files_the_work_started_from() {
    assert_eq!(
        left_by_the_check("cat /task/files/work.txt > started", "started"),
        "to do\n"
    );
}

#[test]
fn the_check_cannot_change_its_task() {
    let evaluated = evaluated(FINISH, "echo changed > /task/statement");
    assert_eq!(
        fs::read_to_string(evaluated.workspace.task("benchmark/t1").join("statement")).unwrap(),
        "Finish the work of t1.\n"
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
    assert_eq!(
        kept(&evaluated(FINISH, "echo out"), "check/stdout"),
        "out\n"
    );
}

#[test]
fn the_standard_error_of_the_check_is_kept_apart_from_that_of_the_harness() {
    assert_eq!(
        kept(&evaluated(FINISH, "echo err >&2"), "check/stderr"),
        "err\n"
    );
}

#[test]
fn an_evaluation_that_ended_leaves_its_outcome_in_its_place() {
    assert_eq!(kept(&evaluated(FINISH, FINISHED), "outcome"), "success\n");
}

#[test]
fn an_evaluation_that_ended_keeps_no_copy_of_the_harness() {
    let evaluated = evaluated(FINISH, FINISHED);
    assert!(
        !evaluated
            .workspace
            .evaluation(0, "benchmark/t1")
            .join("harness")
            .exists()
    );
}

#[test]
fn a_harness_that_runs_past_the_time_limit_times_out() {
    assert!(matches!(
        within(1, "sleep 300", FINISHED).result,
        Ok(EvaluationOutcome::TimedOut)
    ));
}

#[test]
fn a_harness_that_timed_out_is_not_checked() {
    let evaluated = within(1, "sleep 300", "touch checked");
    assert!(
        !evaluated
            .workspace
            .evaluation(0, "benchmark/t1")
            .join("work/checked")
            .exists()
    );
}

#[test]
fn a_check_that_runs_past_what_is_left_of_the_time_limit_times_out() {
    assert!(matches!(
        within(3, FINISH, "sleep 300").result,
        Ok(EvaluationOutcome::TimedOut)
    ));
}

#[test]
fn a_harness_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        evaluated("exit 125", FINISHED).result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn a_check_whose_container_docker_did_not_start_is_an_error() {
    assert!(matches!(
        evaluated(FINISH, "exit 125").result,
        Err(ContainerError::Docker(_))
    ));
}

#[test]
fn what_docker_said_of_a_check_it_did_not_start_is_kept_with_the_check() {
    let evaluated = evaluated(FINISH, "exit 125");
    let kept = evaluated
        .workspace
        .running_evaluation(0, "benchmark/t1")
        .join("check/stderr");
    assert!(matches!(evaluated.result, Err(ContainerError::Docker(said)) if said == kept));
}

#[test]
fn after_an_error_the_directory_of_an_evaluation_is_still_under_running() {
    let evaluated = evaluated("exit 125", FINISHED);
    assert!(
        evaluated
            .workspace
            .running_evaluation(0, "benchmark/t1")
            .join("stdout")
            .is_file()
    );
}

#[test]
fn an_evaluation_of_a_harness_that_is_not_there_is_an_error() {
    let evaluated = evaluated_as(&Evaluation {
        run_command: FINISH,
        check: FINISHED,
        key: None,
        harness: 9,
        time_limit: 30,
    });
    assert!(matches!(evaluated.result, Err(ContainerError::Io(_))));
}
