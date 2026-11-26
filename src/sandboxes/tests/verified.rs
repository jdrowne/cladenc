use std::path::Path;
use std::time::Duration;

use crate::benchmark::{Artifact, EnvironmentConfiguration, HealthCheck, TaskConfiguration};
use crate::genealogy::{Cause, TrialResult};
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{
    self, EMPTIED, HEALTH_CHECK, LAUNCH, LOGS_LINKED, TASK, TESTS_LINKED, TIMER, Tried,
    VERIFIER_EMPTIED,
};
use crate::sandboxes::{Checked, Environment, Launch, Retry, SandboxError, VerifierImage};

const RETRIED: Launch<'static> = Launch {
    max_verifier_health_check_retries: 1,
    ..LAUNCH
};

const VERIFIER_IMAGE: &str = "verifier:t1";
const COPIED: [&str; 2] = ["cp", "--follow-link"];
const CHECK: [&str; 2] = ["exec", HEALTH_CHECK];
const VERIFIER: [&str; 2] = ["exec", "/tests/test.sh"];
const REFUSED: Answer = Answer::Refused("docker is not there");
const REMOVED: &str = "docker rm --force NAME";
const ROOM_MADE: &str = r#"rm -rf "$1" && mkdir -p "${1%/*}/""#;
const GIVEN: &str = "OUTPUTS/running/trial-0-benchmark+t1/given-to-the-verifier";
const BROUGHT_UP: &str = "docker run --rm --pull never --name NAME --label \
                          cladenc.directed-evolution=OUTPUTS --add-host \
                          host.docker.internal:host-gateway";

fn apart(holds_verifier: bool, configuration: TaskConfiguration) -> Environment {
    Environment {
        verifier: Some(VerifierImage {
            name: VERIFIER_IMAGE.to_owned(),
            holds_verifier,
        }),
        ..given::environment(configuration)
    }
}

fn verified_apart(docker: Double) -> Tried {
    verified_with(docker, TaskConfiguration::default())
}

fn verified_with(docker: Double, configuration: TaskConfiguration) -> Tried {
    given::trialled(docker, &LAUNCH, &apart(true, configuration))
}

fn result_of(tried: &Tried) -> Option<TrialResult> {
    tried.result.as_ref().ok().map(|trialled| trialled.result)
}

fn leaving_a_directory() -> Double {
    Double::default().doing(&COPIED, |directory, asked| {
        let left = asked.kept.join("artifacts");
        directory.create_dir_all(&left).unwrap();
        directory.write(left.join("made.txt"), "made\n").unwrap();
    })
}

fn leaving_a_file_no_one_may_read() -> Double {
    Double::default().doing(&COPIED, |directory, asked| {
        let left = asked.kept.join("artifacts");
        directory.create_dir_all(&left).unwrap();
        directory.write(left.join("made.txt"), "made\n").unwrap();
        directory.set_mode(left.join("made.txt"), 0o000).unwrap();
    })
}

fn with_a_kept_file_that_can_no_longer_be_read() -> Double {
    let made = Path::new(&given::kept_published()).join("made.txt");
    leaving_a_directory().doing(&["exec", LOGS_LINKED], move |directory, _| {
        let _ = directory.set_mode(&made, 0o000);
    })
}

fn leaving_a_file() -> Double {
    Double::default().doing(&COPIED, |directory, asked| {
        directory.create_dir_all(&asked.kept).unwrap();
        directory
            .write(asked.kept.join("artifacts"), "made\n")
            .unwrap();
    })
}

fn with_a_verifier_environment() -> TaskConfiguration {
    TaskConfiguration {
        verifier_environment: Some(EnvironmentConfiguration {
            working_directory: Some("/verifying".to_owned()),
            variables: vec![("MODE".to_owned(), "of the verifier".to_owned())],
            cpus: Some(0.5),
            memory_megabytes: Some(512),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    }
}

fn with_a_checked_verifier() -> TaskConfiguration {
    TaskConfiguration {
        verifier_environment: Some(EnvironmentConfiguration {
            health_check: Some(given::checked(1)),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    }
}

fn never_healthy() -> Double {
    Double::default().answering(&CHECK, &[Answer::Exited(1)])
}

#[test]
fn the_sandbox_of_the_task_is_removed_before_its_verifier_sandbox_is_brought_up() {
    let tried = verified_apart(Double::default());
    assert_eq!(
        tried.commands()[10..12],
        [
            REMOVED.to_owned(),
            format!("{BROUGHT_UP} --env-file /dev/stdin --detach verifier:t1 sleep 1200"),
        ]
    );
}

#[test]
fn the_logs_that_harbor_names_are_linked_in_the_verifier_sandbox() {
    let tried = verified_apart(Double::default());
    assert_eq!(
        tried.commands()[13],
        format!(
            "docker exec --workdir / --user 0 NAME sh -c {LOGS_LINKED} sh /logs /var/log/cladenc/harbor"
        )
    );
}

#[test]
fn a_verifier_that_its_image_holds_is_given_no_tests_and_runs_in_its_own_sandbox() {
    let tried = verified_apart(Double::default());
    assert_eq!(
        tried.commands()[14..],
        [
            format!(
                "docker exec --workdir / --user 0 NAME sh -c {LOGS_LINKED} && {VERIFIER_EMPTIED} sh /logs \
                 /var/log/cladenc/harbor /logs/verifier 0"
            ),
            format!(
                "docker exec --env-file /dev/stdin --user 0 NAME sh -c {TIMER} /tests/test.sh 605"
            ),
            "docker cp NAME:/logs/verifier/. -".to_owned(),
            REMOVED.to_owned(),
        ]
    );
}

#[test]
fn a_verifier_that_its_image_does_not_hold_is_given_the_tests_of_the_task() {
    let environment = apart(false, TaskConfiguration::default());
    let tried = given::trialled(Double::default(), &LAUNCH, &environment);
    assert_eq!(
        tried.commands()[14..18],
        [
            format!(
                "docker exec --workdir / --user 0 NAME sh -c {EMPTIED} sh /var/opt/cladenc/harbor/tests"
            ),
            "docker cp OUTPUTS/benchmarks/benchmark/t1/tests/. \
             NAME:/var/opt/cladenc/harbor/tests"
                .to_owned(),
            "docker exec --workdir / --user 0 NAME chown -R 0 /var/opt/cladenc/harbor/tests"
                .to_owned(),
            format!(
                "docker exec --workdir / --user 0 NAME sh -c {TESTS_LINKED} sh /tests \
                 /var/opt/cladenc/harbor/tests"
            ),
        ]
    );
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_decides_the_result_of_the_trial() {
    let tried = verified_apart(Double::default().verifying(|_| true));
    assert_eq!(result_of(&tried), Some(TrialResult::Pass));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_that_runs_past_its_time_limit_timed_out() {
    let docker = Double::default().answering(&VERIFIER, &[Answer::TimedOut]);
    assert_eq!(
        result_of(&verified_apart(docker)),
        Some(TrialResult::Inconclusive(Cause::TimedOut))
    );
}

#[test]
fn what_docker_says_when_it_brings_up_a_verifier_sandbox_is_kept_with_the_verifier() {
    let tried = verified_apart(Double::default());
    assert_eq!(
        tried.docker.asked()[11].kept,
        tried.running().join("verifier/stderr")
    );
}

#[test]
fn a_verifier_sandbox_is_given_the_settings_of_the_verifier_environment() {
    let tried = verified_with(Double::default(), with_a_verifier_environment());
    assert_eq!(
        tried.commands()[11],
        format!(
            "{BROUGHT_UP} --workdir /verifying --cpus 0.5 --memory 512m --env-file /dev/stdin \
             --detach verifier:t1 sleep 1200"
        )
    );
}

#[test]
fn a_verifier_sandbox_is_given_the_variables_of_the_verifier_environment() {
    let tried = verified_with(Double::default(), with_a_verifier_environment());
    assert_eq!(tried.docker.asked()[11].input, "MODE=of the verifier\n");
}

#[test]
fn a_directory_the_harness_left_as_artifacts_is_given_to_the_verifier_at_the_same_path() {
    let tried = verified_apart(leaving_a_directory());
    assert_eq!(
        tried.commands()[14..17],
        [
            format!("docker exec --workdir / --user 0 NAME sh -c {EMPTIED} sh /logs/artifacts"),
            format!("docker cp {GIVEN}/. NAME:/logs/artifacts"),
            "docker exec --workdir / --user 0 NAME chown -R 0 /logs/artifacts".to_owned(),
        ]
    );
}

#[test]
fn a_file_the_harness_left_as_artifacts_is_given_to_the_verifier_at_the_same_path() {
    let tried = verified_apart(leaving_a_file());
    assert_eq!(
        tried.commands()[14..17],
        [
            format!("docker exec --workdir / --user 0 NAME sh -c {ROOM_MADE} sh /logs/artifacts"),
            format!("docker cp {GIVEN} NAME:/logs/artifacts"),
            "docker exec --workdir / --user 0 NAME chown -R 0 /logs/artifacts".to_owned(),
        ]
    );
}

#[test]
fn a_path_the_task_names_to_be_kept_is_given_to_the_verifier_before_the_published_place() {
    let configuration = TaskConfiguration {
        artifacts: vec![Artifact {
            source: "/app/out".to_owned(),
            destination: "out".to_owned(),
            exclude: Vec::new(),
        }],
        ..TaskConfiguration::default()
    };
    let tried = verified_with(leaving_a_directory(), configuration);
    assert_eq!(
        tried.commands()[16],
        format!("docker cp {GIVEN}/. NAME:/app/out")
    );
}

#[test]
fn what_is_given_to_the_verifier_is_owned_by_the_user_the_task_names_for_the_verifier() {
    let configuration = TaskConfiguration {
        verifier_user: Some("judge".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = verified_with(leaving_a_directory(), configuration);
    assert_eq!(
        tried.commands()[16],
        "docker exec --workdir / --user 0 NAME chown -R judge /logs/artifacts"
    );
}

#[test]
fn the_copy_that_is_given_to_the_verifier_is_not_kept_with_the_trial() {
    let tried = verified_apart(leaving_a_directory());
    let trial = tried.outputs_directory.trial(0, TASK);
    assert!(!tried.directory.exists(trial.join("given-to-the-verifier")));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_runs_as_the_user_of_its_own_image() {
    let users = [
        Answer::Printed("task\n/\n"),
        Answer::Printed("checker\n/\n"),
    ];
    let tried = verified_apart(Double::default().answering(&["inspect"], &users));
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["checker"]
    );
}

#[test]
fn the_health_check_of_a_verifier_runs_in_its_sandbox_before_anything_else() {
    let tried = verified_with(Double::default(), with_a_checked_verifier());
    assert_eq!(
        tried.commands()[13],
        format!("docker exec --env-file /dev/stdin --user 0 NAME sh -c {TIMER} {HEALTH_CHECK} 35")
    );
}

#[test]
fn what_the_health_check_of_a_verifier_prints_is_kept_with_the_verifier() {
    let tried = verified_with(Double::default(), with_a_checked_verifier());
    assert_eq!(
        tried.docker.asked()[13].kept,
        tried.running().join("verifier/health-check")
    );
}

#[test]
fn a_verifier_sandbox_with_a_health_check_stays_for_the_longest_the_check_can_take() {
    let tried = verified_with(Double::default(), with_a_checked_verifier());
    assert_eq!(tried.commands()[11].rsplit(' ').next(), Some("1260"));
}

#[test]
fn a_verifier_whose_health_check_does_not_pass_is_not_run() {
    let tried = verified_with(never_healthy(), with_a_checked_verifier());
    let printed = tried.running().join("verifier/health-check");
    assert!(matches!(
        &tried.result,
        Err(SandboxError::VerifierHealthCheck { tries: 1, kept }) if *kept == printed
    ));
}

#[test]
fn a_trial_whose_verifier_is_never_run_keeps_no_result() {
    let tried = verified_with(never_healthy(), with_a_checked_verifier());
    assert!(
        !tried
            .directory
            .exists(tried.outputs_directory.trial(0, TASK))
    );
}

#[test]
fn no_retry_of_the_health_check_of_a_verifier_is_told_where_none_is_allowed() {
    let tried = verified_with(never_healthy(), with_a_checked_verifier());
    assert!(tried.retries.is_empty());
}

#[test]
fn a_health_check_of_a_verifier_that_docker_does_not_run_names_the_file_that_holds_what_it_said() {
    let docker = Double::default().answering(&CHECK, &[Answer::Exited(125)]);
    let tried = verified_with(docker, with_a_checked_verifier());
    let said = tried.running().join("verifier/health-check/stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_verifier_sandbox_whose_health_check_does_not_pass_is_removed() {
    let tried = verified_with(never_healthy(), with_a_checked_verifier());
    assert_eq!(tried.commands()[14..], [REMOVED]);
}

#[test]
fn a_verifier_whose_health_check_does_not_pass_is_brought_up_again_as_often_as_is_allowed() {
    let environment = apart(true, with_a_checked_verifier());
    let tried = given::trialled(never_healthy(), &RETRIED, &environment);
    let brought_up = tried
        .commands()
        .iter()
        .filter(|command| command.contains("--detach verifier:t1"))
        .count();
    assert_eq!(brought_up, 2);
}

#[test]
fn each_time_a_verifier_is_brought_up_again_for_its_health_check_is_told() {
    let environment = apart(true, with_a_checked_verifier());
    let tried = given::trialled(never_healthy(), &RETRIED, &environment);
    assert_eq!(
        tried.retries,
        [Retry {
            checked: Checked::Verifier,
            number: 1,
            most: 1,
            wait: Duration::ZERO,
        }]
    );
}

#[test]
fn the_wait_before_a_verifier_is_brought_up_again_starts_at_the_interval_of_its_check_and_doubles()
{
    let configuration = TaskConfiguration {
        verifier_environment: Some(EnvironmentConfiguration {
            health_check: Some(HealthCheck {
                interval: Duration::from_millis(1),
                ..given::checked(1)
            }),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    };
    let launch = Launch {
        max_verifier_health_check_retries: 2,
        ..LAUNCH
    };
    let tried = given::trialled(never_healthy(), &launch, &apart(true, configuration));
    let waits: Vec<Duration> = tried.retries.iter().map(|retry| retry.wait).collect();
    assert_eq!(waits, [Duration::from_millis(1), Duration::from_millis(2)]);
}

#[test]
fn a_verifier_whose_health_check_never_passes_names_how_often_it_was_tried() {
    let environment = apart(true, with_a_checked_verifier());
    let tried = given::trialled(never_healthy(), &RETRIED, &environment);
    assert!(matches!(
        tried.result,
        Err(SandboxError::VerifierHealthCheck { tries: 2, .. })
    ));
}

#[test]
fn a_verifier_whose_health_check_passes_once_it_is_brought_up_again_is_run() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&CHECK, &[Answer::Exited(1), Answer::Exited(0)]);
    let environment = apart(true, with_a_checked_verifier());
    let tried = given::trialled(docker, &RETRIED, &environment);
    assert_eq!(result_of(&tried), Some(TrialResult::Pass));
}

#[test]
fn a_verifier_whose_sandbox_docker_does_not_bring_up_names_the_file_that_holds_what_it_said() {
    let detached = [Answer::Exited(0), Answer::Exited(1)];
    let tried = verified_apart(Double::default().answering(&["--detach"], &detached));
    let said = tried.running().join("verifier/stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_trial_whose_verifier_logs_cannot_be_copied_out_of_its_verifier_sandbox_is_given_no_result() {
    let kept = given::kept_verifier_logs();
    let docker = Double::default().answering(&["cp", kept.as_str()], &[REFUSED]);
    assert!(matches!(
        verified_apart(docker).result,
        Err(SandboxError::NotCopiedOut { .. })
    ));
}

#[test]
fn no_verifier_sandbox_is_brought_up_where_what_it_would_be_given_was_not_copied_out() {
    let tried = verified_apart(Double::default().answering(&COPIED, &[REFUSED]));
    assert_eq!(
        tried
            .commands()
            .iter()
            .filter(|command| command.starts_with(BROUGHT_UP))
            .count(),
        1
    );
}

#[test]
fn a_trial_whose_verifier_would_be_given_part_of_a_kept_path_is_given_no_result() {
    assert!(matches!(
        verified_apart(with_a_kept_file_that_can_no_longer_be_read()).result,
        Err(SandboxError::NotGivenToVerifier { left, .. }) if left == "/logs/artifacts"
    ));
}

#[test]
fn a_file_no_one_may_read_at_a_kept_path_is_given_to_the_verifier() {
    let tried = verified_apart(leaving_a_file_no_one_may_read());
    assert!(
        tried
            .commands()
            .iter()
            .any(|command| *command == format!("docker cp {GIVEN}/. NAME:/logs/artifacts"))
    );
}

#[test]
fn a_file_no_one_may_read_at_a_kept_path_is_kept_as_a_file_its_owner_can_read() {
    let tried = verified_apart(leaving_a_file_no_one_may_read());
    let trial = tried.outputs_directory.trial(0, TASK);
    let kept = trial.join("artifacts/logs/artifacts/made.txt");
    assert_eq!(tried.directory.mode(kept) & 0o777, 0o400);
}

#[test]
fn a_kept_path_that_was_not_copied_in_full_is_told_with_the_file_that_was_not_copied() {
    let tried = verified_apart(with_a_kept_file_that_can_no_longer_be_read());
    let kept = tried.running().join("artifacts/logs/artifacts/made.txt");
    assert!(matches!(
        &tried.result,
        Err(SandboxError::NotGivenToVerifier { error, .. })
            if error.to_string().starts_with(&format!("{}: ", kept.display()))
    ));
}

#[test]
fn no_part_of_a_kept_path_that_was_not_copied_in_full_is_put_in_the_verifier_sandbox() {
    let tried = verified_apart(with_a_kept_file_that_can_no_longer_be_read());
    assert!(
        !tried
            .commands()
            .iter()
            .any(|command| command.starts_with(&format!("docker cp {GIVEN}")))
    );
}

#[test]
fn no_verifier_is_run_on_part_of_a_kept_path() {
    let tried = verified_apart(with_a_kept_file_that_can_no_longer_be_read());
    assert!(
        !tried
            .commands()
            .iter()
            .any(|command| command.contains("/tests/test.sh"))
    );
}

#[test]
fn a_copy_for_the_verifier_that_was_not_made_in_full_is_not_left_behind() {
    let tried = verified_apart(with_a_kept_file_that_can_no_longer_be_read());
    let given = tried.running().join("given-to-the-verifier");
    assert!(!tried.directory.exists(given));
}

#[test]
fn a_trial_whose_logs_cannot_be_linked_in_its_verifier_sandbox_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", LOGS_LINKED], &[REFUSED]);
    assert!(matches!(
        verified_apart(docker).result,
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn a_trial_whose_artifacts_cannot_be_given_to_its_verifier_is_not_carried_out() {
    let docker = leaving_a_directory().answering(&["chown", "/logs/artifacts"], &[REFUSED]);
    assert!(matches!(
        verified_apart(docker).result,
        Err(SandboxError::Io(_))
    ));
}
