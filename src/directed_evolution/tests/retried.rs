use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{Settings, Started};
use crate::sandboxes::SandboxError;
use crate::sandboxes::tests::double::{Answer, Double};

const CHECKED: &str = "[environment.healthcheck]\ncommand = \"test -e /ready\"\ninterval_sec = \
                       0.0\nretries = 1\n";

fn with_a_health_check(directory: &Memory, settings: &mut Settings) {
    settings.max_task_health_check_retries = 1;
    given::configured(directory, "t1", CHECKED);
}

fn brought_up_again() -> Evolved {
    let answers = [Answer::Exited(1), Answer::Exited(0)];
    let docker = Double::default().answering(&["exec", "test -e /ready"], &answers);
    given::started(docker, with_a_health_check)
}

#[test]
fn a_task_that_is_brought_up_again_for_its_health_check_is_told() {
    let said = brought_up_again().said;
    assert!(said.contains(
        "the trial of harness 0 on task benchmark/t1 is not carried out yet: the health check \
         of the task did not pass"
    ));
}

#[test]
fn a_task_that_is_brought_up_again_is_told_with_the_number_of_the_retry() {
    let said = brought_up_again().said;
    assert!(said.contains("retry 1 of 1"));
}

#[test]
fn a_task_that_is_brought_up_again_gets_a_new_sandbox() {
    assert_eq!(brought_up_again().count_of("--detach"), 3);
}

#[test]
fn a_directed_evolution_goes_on_once_the_health_check_of_a_task_passes() {
    let best = brought_up_again()
        .result
        .ok()
        .map(|found| found.best_harness);
    assert_eq!(best, Some(0));
}

const VERIFIER_CHECKED: &str = "[verifier.environment.healthcheck]\ncommand = \"test -e \
                              /verified\"\ninterval_sec = 0.0\nretries = 1\n";
const STARTED: &str = r#"{"record":"trial_started","harness":0,"task":"benchmark/t1"}"#;
const FINISHED: &str = r#"{"record":"trial_finished","harness":0,"task":"benchmark/t1""#;
const STARTED_AGAIN: &str = "cladenc: started again the trial of harness 0 on task benchmark/t1\n";

fn with_a_health_check_of_the_verifier(directory: &Memory, settings: &mut Settings) {
    settings.max_verifier_health_check_retries = 1;
    given::configured(directory, "t1", VERIFIER_CHECKED);
}

fn never_ready() -> Evolved {
    let docker = Double::default().answering(&["exec", "test -e /ready"], &[Answer::Exited(1)]);
    given::started(docker, with_a_health_check)
}

fn with_a_verifier_that_is_never_ready() -> Evolved {
    let docker = Double::default().answering(&["exec", "test -e /verified"], &[Answer::Exited(1)]);
    given::started(docker, with_a_health_check_of_the_verifier)
}

fn journaled(evolved: &Evolved) -> [bool; 2] {
    let journal = evolved.journal();
    [STARTED, FINISHED].map(|record| journal.contains(record))
}

#[test]
fn a_task_that_is_never_ready_stops_the_directed_evolution_naming_the_trial_and_the_tries() {
    assert!(matches!(
        never_ready().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { harness: 0, task },
            error: SandboxError::TaskHealthCheck { tries: 2, .. },
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_harness_that_was_never_started_is_said_with_the_tries_and_where_the_health_check_is_kept() {
    let evolved = never_ready();
    assert!(evolved.said.contains(&format!(
        "cladenc: the trial of harness 0 on task benchmark/t1 was not carried out: the health \
         check of the task did not pass in 2 tries, so the harness was not started, and {} holds \
         what the health check printed the last time it ran\n",
        evolved
            .outputs()
            .join("running/trial-0-benchmark+t1/health-check")
            .display()
    )));
}

#[test]
fn a_trial_whose_harness_was_never_started_is_not_said_to_be_inconclusive() {
    let said = never_ready().said;
    assert!(!said.contains("on task benchmark/t1 was inconclusive"));
}

#[test]
fn a_trial_whose_harness_was_never_started_is_journaled_as_started_and_not_as_finished() {
    assert_eq!(journaled(&never_ready()), [true, false]);
}

#[test]
fn a_trial_whose_harness_was_never_started_is_carried_out_again_when_resumed() {
    let resumed = given::resumed(never_ready(), Double::default());
    assert!(resumed.said.contains(STARTED_AGAIN));
}

#[test]
fn a_directed_evolution_resumed_once_the_task_is_ready_runs_to_its_end() {
    let resumed = given::resumed(never_ready(), Double::default());
    assert!(resumed.result.is_ok());
}

#[test]
fn a_verifier_that_is_never_ready_stops_the_directed_evolution_naming_the_trial_and_the_tries() {
    assert!(matches!(
        with_a_verifier_that_is_never_ready().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Trial { harness: 0, task },
            error: SandboxError::VerifierHealthCheck { tries: 2, .. },
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_verifier_that_is_brought_up_again_for_its_health_check_is_told_with_the_number_of_the_retry() {
    let said = with_a_verifier_that_is_never_ready().said;
    assert!(said.contains(
        "cladenc: the trial of harness 0 on task benchmark/t1 is not verified yet: the health \
         check of its verifier sandbox did not pass, and cladenc tries again in a new \
         sandbox after 0 seconds, retry 1 of 1\n"
    ));
}

#[test]
fn a_verifier_that_was_never_run_is_said_with_the_tries_and_where_its_health_check_is_kept() {
    let evolved = with_a_verifier_that_is_never_ready();
    assert!(evolved.said.contains(&format!(
        "cladenc: the trial of harness 0 on task benchmark/t1 was not verified: the health \
         check of its verifier sandbox did not pass in 2 tries, so the verifier was not \
         run, and {} holds what the health check printed the last time it ran\n",
        evolved
            .outputs()
            .join("running/trial-0-benchmark+t1/verifier/health-check")
            .display()
    )));
}

#[test]
fn a_trial_whose_verifier_was_never_run_is_not_said_to_be_inconclusive() {
    let said = with_a_verifier_that_is_never_ready().said;
    assert!(!said.contains("on task benchmark/t1 was inconclusive"));
}

#[test]
fn a_trial_whose_verifier_was_never_run_is_journaled_as_started_and_not_as_finished() {
    assert_eq!(
        journaled(&with_a_verifier_that_is_never_ready()),
        [true, false]
    );
}

#[test]
fn a_trial_whose_verifier_was_never_run_is_carried_out_again_when_resumed() {
    let resumed = given::resumed(with_a_verifier_that_is_never_ready(), Double::default());
    assert!(resumed.said.contains(STARTED_AGAIN));
}

#[test]
fn a_directed_evolution_resumed_once_the_verifier_is_ready_runs_to_its_end() {
    let resumed = given::resumed(with_a_verifier_that_is_never_ready(), Double::default());
    assert!(resumed.result.is_ok());
}

#[test]
fn what_the_health_check_of_a_task_that_was_never_ready_printed_is_kept_where_cladenc_says() {
    let evolved = never_ready();
    let kept = evolved
        .outputs()
        .join("running/trial-0-benchmark+t1/health-check");
    assert!(evolved.files().is_dir(kept));
}

#[test]
fn what_the_health_check_of_a_verifier_that_was_never_run_printed_is_kept_where_cladenc_says() {
    let evolved = with_a_verifier_that_is_never_ready();
    let kept = evolved
        .outputs()
        .join("running/trial-0-benchmark+t1/verifier/health-check");
    assert!(evolved.files().is_dir(kept));
}
