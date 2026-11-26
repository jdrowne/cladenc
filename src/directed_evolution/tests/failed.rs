use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::journal::Started;
use crate::sandboxes::SandboxError;
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::kept_verifier_logs;

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const DIVERSIFICATION: [&str; 2] = ["run", "run a harness"];
const FIRST_TRIAL_STARTED: &str = r#"{"record":"trial_started","harness":0,"task":"benchmark/t1"}"#;
const STARTED_AGAIN: &str = "cladenc: started again the trial of harness 0 on task benchmark/t1\n";

fn refused() -> Evolved {
    let refused = [Answer::Refused("docker is not there")];
    let docker = Double::default().answering(&["--detach"], &refused);
    given::started(docker, given::unchanged)
}

fn not_copied_out() -> Evolved {
    let kept = kept_verifier_logs();
    let refused = [Answer::Refused("no space left on device")];
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&["cp", kept.as_str()], &refused);
    given::started(docker, given::unchanged)
}

fn stopped() -> Evolved {
    let stop = Arc::new(AtomicBool::new(false));
    let asked = Arc::clone(&stop);
    let docker = Double::default()
        .answering(&HARNESS, &[Answer::UntilStopped])
        .doing(&HARNESS, move |_, _| asked.store(true, Ordering::SeqCst));
    given::started_until(docker, given::unchanged, &stop)
}

#[test]
fn a_trial_that_cannot_be_carried_out_ends_the_directed_evolution_with_its_cause() {
    assert!(matches!(
        refused().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { harness: 0, task },
            error: SandboxError::Io(_),
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_trial_that_cannot_be_carried_out_is_told_with_its_cause() {
    let said = refused().said;
    assert!(said.contains(
        "the trial of harness 0 on task benchmark/t1 could not be carried out: docker is not \
         there"
    ));
}

#[test]
fn a_directed_evolution_tells_that_it_stops_when_a_trial_cannot_be_carried_out() {
    let said = refused().said;
    assert!(said.contains("stopping the directed evolution, with"));
}

#[test]
fn nothing_is_started_after_a_trial_that_cannot_be_carried_out() {
    assert_eq!(refused().count_of("--detach"), 1);
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_while_a_harness_runs_was_asked_to_stop() {
    assert!(matches!(
        stopped().result,
        Err(DirectedEvolutionError::Asked)
    ));
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_tells_so() {
    let said = stopped().said;
    assert!(said.contains("asked to stop, with"));
}

#[test]
fn nothing_is_started_after_a_directed_evolution_is_asked_to_stop() {
    assert_eq!(stopped().count_of("--detach"), 1);
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_0_or_125_does_not_stop_the_directed_evolution() {
    let docker = Double::default().answering(&["run a harness"], &[Answer::Exited(2)]);
    assert!(given::started(docker, given::unchanged).result.is_ok());
}

#[test]
fn a_trial_that_cannot_be_carried_out_is_journaled_as_started_and_not_as_finished() {
    assert_eq!(
        refused().journal().lines().last(),
        Some(FIRST_TRIAL_STARTED)
    );
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_journals_nothing_for_the_action_that_was_running() {
    assert_eq!(
        stopped().journal().lines().last(),
        Some(FIRST_TRIAL_STARTED)
    );
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_says_so_last_with_the_sandboxes_running() {
    let said = stopped().said;
    assert!(said.ends_with("cladenc: asked to stop, with 1 sandbox running\n"));
}

#[test]
fn a_diversification_whose_sandbox_docker_does_not_start_is_told_when_it_happens() {
    let docker = Double::default().answering(&DIVERSIFICATION, &[Answer::Exited(125)]);
    let said = given::started(docker, given::unchanged).said;
    assert!(said.contains(
        "cladenc: the diversification of harness 1 by harness 0 could not be carried out: Docker \
         did not start its sandbox"
    ));
}

#[test]
fn a_harness_that_runs_past_the_time_limit_of_its_task_is_said_to_have_been_stopped_at_it() {
    let docker = Double::default().answering(&HARNESS, &[Answer::TimedOut]);
    let said = given::started(docker, given::unchanged).said;
    assert!(said.contains(
        "cladenc: the trial of harness 0 on task benchmark/t1 was inconclusive, after the \
         harness was stopped at its time limit, as the verifier left no verdict that can be \
         read: what the trial left is kept in "
    ));
}

#[test]
fn a_directed_evolution_whose_harnesses_all_run_out_of_time_runs_to_its_end() {
    let docker = Double::default().answering(&HARNESS, &[Answer::TimedOut]);
    assert!(given::started(docker, given::unchanged).result.is_ok());
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_ends_the_directed_evolution_with_its_cause() {
    assert!(matches!(
        not_copied_out().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { harness: 0, task },
            error: SandboxError::NotCopiedOut { .. },
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_is_told_with_what_docker_said() {
    let said = not_copied_out().said;
    assert!(said.contains(
        "cladenc: the trial of harness 0 on task benchmark/t1 was not given a result: what its \
         verifier left in /logs/verifier could not be copied out of its sandbox: no space left on \
         device\n"
    ));
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_is_not_said_to_be_inconclusive() {
    let said = not_copied_out().said;
    assert!(!said.contains("on task benchmark/t1 was inconclusive"));
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_is_journaled_as_started_and_not_as_finished() {
    assert_eq!(
        not_copied_out().journal().lines().last(),
        Some(FIRST_TRIAL_STARTED)
    );
}

#[test]
fn a_trial_whose_verifier_logs_were_not_copied_out_is_carried_out_again_when_resumed() {
    let resumed = given::resumed(not_copied_out(), Double::default());
    assert!(resumed.said.contains(STARTED_AGAIN));
}

#[test]
fn a_directed_evolution_resumed_once_the_verifier_logs_can_be_copied_out_runs_to_its_end() {
    let resumed = given::resumed(not_copied_out(), Double::default());
    assert!(resumed.result.is_ok());
}
