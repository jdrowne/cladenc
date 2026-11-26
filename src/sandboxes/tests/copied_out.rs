use crate::dataset::TaskConfiguration;
use crate::genealogy::{Cause, TrialResult};
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{self, LAUNCH, TASK, Tried};
use crate::sandboxes::{Copied, SandboxError};

const VERIFIER: [&str; 2] = ["exec", "/tests/test.sh"];
const NO_DIRECTORY: &str = r#"[ -d "$1" ] || echo none"#;
const ASKED: [&str; 2] = ["exec", NO_DIRECTORY];
const NOT_COPIED: Answer = Answer::Refused("the copy failed");
const NONE: Answer = Answer::Printed("none\n");
const WORKING_IN_APP: Answer = Answer::Printed("\n/app\n");
const REMOVED: &str = "docker rm --force NAME";

fn not_copying() -> Double {
    let kept = given::kept_verifier_logs();
    Double::default().answering(&["cp", kept.as_str()], &[NOT_COPIED])
}

fn not_copying_the_work() -> Double {
    let kept = given::kept_work();
    Double::default()
        .answering(&["inspect"], &[WORKING_IN_APP])
        .answering(&["cp", kept.as_str()], &[NOT_COPIED])
}

fn verifier_was_run(tried: &Tried) -> bool {
    tried
        .commands()
        .iter()
        .any(|command| command.contains(" /tests/test.sh "))
}

fn trialled_by(docker: Double) -> Tried {
    let environment = given::environment(TaskConfiguration::default());
    given::trialled(docker, &LAUNCH, &environment)
}

fn result_of(tried: &Tried) -> Option<TrialResult> {
    tried.result.as_ref().ok().map(|trialled| trialled.result)
}

fn reason_of(tried: &Tried) -> Option<String> {
    match &tried.result {
        Err(SandboxError::NotCopiedOut { error, .. }) => Some(error.to_string()),
        _ => None,
    }
}

#[test]
fn a_trial_whose_verifier_logs_cannot_be_copied_out_of_the_sandbox_is_given_no_result() {
    let tried = trialled_by(not_copying());
    assert!(matches!(
        tried.result,
        Err(SandboxError::NotCopiedOut {
            left: Copied::VerifierLogs,
            ..
        })
    ));
}

#[test]
fn what_docker_said_of_the_copy_is_the_reason_the_verifier_logs_were_not_copied_out() {
    let tried = trialled_by(not_copying());
    assert_eq!(reason_of(&tried).as_deref(), Some("the copy failed"));
}

#[test]
fn a_verdict_that_a_failed_copy_brought_out_all_the_same_is_not_read() {
    let tried = trialled_by(not_copying().verifying(|_| true));
    assert!(matches!(
        tried.result,
        Err(SandboxError::NotCopiedOut { .. })
    ));
}

#[test]
fn the_sandbox_is_asked_whether_the_verifier_left_a_directory_once_the_copy_has_failed() {
    let tried = trialled_by(not_copying());
    assert_eq!(
        tried.commands()[17],
        format!("docker exec --workdir / --user 0 NAME sh -c {NO_DIRECTORY} sh /logs/verifier")
    );
}

#[test]
fn a_verifier_that_left_no_directory_to_copy_left_no_verdict() {
    let docker = not_copying().answering(&ASKED, &[NONE]);
    assert_eq!(
        result_of(&trialled_by(docker)),
        Some(TrialResult::Inconclusive(Cause::NoVerdict))
    );
}

#[test]
fn a_sandbox_that_cannot_be_asked_what_the_verifier_left_gives_the_reason_of_the_copy() {
    let docker = not_copying().answering(&ASKED, &[Answer::Refused("the sandbox is gone")]);
    assert_eq!(
        reason_of(&trialled_by(docker)).as_deref(),
        Some("the copy failed")
    );
}

#[test]
fn a_sandbox_that_answers_anything_else_is_not_taken_to_hold_no_directory() {
    let docker = not_copying().answering(&ASKED, &[Answer::Printed("nonesuch\n")]);
    assert!(matches!(
        trialled_by(docker).result,
        Err(SandboxError::NotCopiedOut { .. })
    ));
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_stays_among_the_running_ones() {
    let tried = trialled_by(not_copying());
    assert!(tried.directory.is_dir(tried.running()));
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_keeps_no_result() {
    let tried = trialled_by(not_copying().verifying(|_| true));
    assert!(
        !tried
            .directory
            .exists(tried.outputs_directory.trial(0, TASK))
    );
}

#[test]
fn the_sandbox_of_a_trial_whose_verifier_logs_were_not_copied_out_is_removed() {
    let tried = trialled_by(not_copying());
    assert_eq!(tried.commands().pop().as_deref(), Some(REMOVED));
}

#[test]
fn a_verifier_that_timed_out_gives_no_result_where_its_logs_cannot_be_copied_out() {
    let docker = not_copying().answering(&VERIFIER, &[Answer::TimedOut]);
    assert!(matches!(
        trialled_by(docker).result,
        Err(SandboxError::NotCopiedOut {
            left: Copied::VerifierLogs,
            ..
        })
    ));
}

#[test]
fn the_sandbox_of_a_verifier_that_timed_out_is_asked_whether_the_verifier_left_a_directory() {
    let docker = not_copying().answering(&VERIFIER, &[Answer::TimedOut]);
    assert_eq!(
        trialled_by(docker).commands()[17],
        format!("docker exec --workdir / --user 0 NAME sh -c {NO_DIRECTORY} sh /logs/verifier")
    );
}

#[test]
fn a_verifier_that_ran_past_its_time_limit_and_left_no_directory_timed_out() {
    let docker = not_copying()
        .answering(&VERIFIER, &[Answer::TimedOut])
        .answering(&ASKED, &[NONE]);
    assert_eq!(
        result_of(&trialled_by(docker)),
        Some(TrialResult::Inconclusive(Cause::TimedOut))
    );
}

#[test]
fn a_trial_whose_work_cannot_be_copied_out_of_the_sandbox_is_given_no_result() {
    let tried = trialled_by(not_copying_the_work());
    assert!(matches!(
        &tried.result,
        Err(SandboxError::NotCopiedOut { left: Copied::Work(directory), .. }) if directory == "/app"
    ));
}

#[test]
fn what_docker_said_of_the_copy_is_the_reason_the_work_was_not_copied_out() {
    let tried = trialled_by(not_copying_the_work());
    assert_eq!(reason_of(&tried).as_deref(), Some("the copy failed"));
}

#[test]
fn the_verifier_of_a_trial_whose_work_was_not_copied_out_is_not_run() {
    let tried = trialled_by(not_copying_the_work());
    assert!(!verifier_was_run(&tried));
}

#[test]
fn a_trial_whose_work_was_not_copied_out_keeps_no_result() {
    let tried = trialled_by(not_copying_the_work());
    assert!(
        !tried
            .directory
            .exists(tried.outputs_directory.trial(0, TASK))
    );
}

#[test]
fn the_sandbox_of_a_trial_whose_work_was_not_copied_out_is_removed() {
    let tried = trialled_by(not_copying_the_work());
    assert_eq!(tried.commands().pop().as_deref(), Some(REMOVED));
}

#[test]
fn a_trial_goes_on_where_the_harness_left_no_working_directory() {
    let docker = not_copying_the_work()
        .verifying(|_| true)
        .answering(&ASKED, &[NONE]);
    assert_eq!(result_of(&trialled_by(docker)), Some(TrialResult::Pass));
}
