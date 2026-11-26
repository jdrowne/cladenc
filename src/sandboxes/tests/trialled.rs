use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use crate::benchmark::{Artifact, CollectCommand, TaskConfiguration};
use crate::genealogy::{Cause, TrialResult};
use crate::model::Model;
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{
    self, ARTIFACTS_MADE, EMPTIED, LAUNCH, LOGS_LINKED, TASK, TESTS_LINKED, TIMER, Tried,
    VERIFIER_EMPTIED,
};
use crate::sandboxes::{self, HarnessEnding, Launch, SandboxError, Trial};

const KEYED: Launch<'static> = Launch {
    model: Model {
        key: Some(test_inputs::MODEL_KEY),
        ..LAUNCH.model
    },
    ..LAUNCH
};

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const VERIFIER: [&str; 2] = ["exec", "/tests/test.sh"];
const COLLECT: [&str; 2] = ["exec", "ls /app"];
const REFUSED: Answer = Answer::Refused("docker is not there");
const REMOVED: &str = "docker rm --force NAME";
const TOLD: &str = "--env CLADENC_HARNESS --env CLADENC_INSTRUCTION --env CLADENC_MODEL \
                    --env CLADENC_MODEL_URL";

fn trialled_by(docker: Double) -> Tried {
    trialled_with(docker, TaskConfiguration::default())
}

fn trialled_with(docker: Double, configuration: TaskConfiguration) -> Tried {
    given::trialled(docker, &LAUNCH, &given::environment(configuration))
}

fn result_of(tried: &Tried) -> Option<TrialResult> {
    tried.result.as_ref().ok().map(|trialled| trialled.result)
}

fn ending_of(tried: &Tried) -> Option<HarnessEnding> {
    tried.result.as_ref().ok().map(|trialled| trialled.ending)
}

fn kept(tried: &Tried, file: &str) -> Option<String> {
    tried
        .directory
        .read_to_string(tried.outputs_directory.trial(0, TASK).join(file))
        .ok()
}

fn last_word_of(command: &str) -> Option<&str> {
    command.rsplit(' ').next()
}

fn with_a_user_of_the_image() -> Double {
    Double::default().answering(&["inspect"], &[Answer::Printed("builder\n/\n")])
}

fn taking(needles: &[&str], milliseconds: u64) -> Double {
    Double::default().doing(needles, move |_, _| {
        thread::sleep(Duration::from_millis(milliseconds));
    })
}

fn seconds_of(tried: &Tried) -> Option<f64> {
    tried.result.as_ref().ok().map(|trialled| trialled.seconds)
}

fn collecting(users: &[Option<&str>]) -> TaskConfiguration {
    TaskConfiguration {
        collect_commands: users
            .iter()
            .map(|user| CollectCommand {
                command: "ls /app".to_owned(),
                user: user.map(str::to_owned),
                time_limit: Duration::from_secs(30),
            })
            .collect(),
        ..TaskConfiguration::default()
    }
}

#[test]
fn a_trial_brings_up_a_sandbox_of_the_task_that_stays_past_the_harness_and_the_verifier() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[0],
        "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
         --add-host host.docker.internal:host-gateway --env-file /dev/stdin --detach \
         debian:stable-slim sleep 4294968495"
    );
}

#[test]
fn the_source_code_of_the_harness_is_put_in_the_sandbox_and_owned_by_who_runs_it() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[2..5],
        [
            format!("docker exec --user 0 NAME sh -c {EMPTIED} sh /var/opt/cladenc/harness"),
            "docker cp OUTPUTS/harnesses/0/. NAME:/var/opt/cladenc/harness".to_owned(),
            "docker exec --user 0 NAME chown -R 0 /var/opt/cladenc/harness".to_owned(),
        ]
    );
}

#[test]
fn the_harness_is_given_a_place_for_its_artifacts_before_it_runs() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[5],
        format!(
            "docker exec --user 0 NAME sh -c {LOGS_LINKED} && {ARTIFACTS_MADE} sh /logs \
             /var/log/cladenc/harbor /logs/artifacts 0"
        )
    );
}

#[test]
fn the_harness_runs_by_its_run_command_as_the_user_of_the_image_with_no_time_limit() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[7],
        format!(
            "docker exec --env-file /dev/stdin {TOLD} --user 0 NAME sh -c {TIMER} run a harness \
             4294967300"
        )
    );
}

#[test]
fn what_changed_in_the_sandbox_is_asked_for_before_and_after_the_harness_runs() {
    let commands = trialled_by(Double::default()).commands();
    assert_eq!(
        [commands[6].as_str(), commands[8].as_str()],
        ["docker diff NAME"; 2]
    );
}

#[test]
fn what_the_harness_left_in_the_published_place_for_artifacts_is_copied_out() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[9],
        "docker cp --follow-link NAME:/logs/artifacts -"
    );
}

#[test]
fn what_the_harness_left_at_a_path_is_unpacked_beside_the_artifacts_of_the_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.docker.asked()[9].kept,
        tried.running().join("taken-out-of-the-sandbox")
    );
}

#[test]
fn the_tests_of_the_task_are_put_in_the_sandbox_only_once_the_harness_has_ended() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[10..14],
        [
            format!("docker exec --user 0 NAME sh -c {EMPTIED} sh /var/opt/cladenc/harbor/tests"),
            "docker cp OUTPUTS/benchmarks/benchmark/t1/tests/. \
             NAME:/var/opt/cladenc/harbor/tests"
                .to_owned(),
            "docker exec --user 0 NAME chown -R 0 /var/opt/cladenc/harbor/tests".to_owned(),
            format!(
                "docker exec --user 0 NAME sh -c {TESTS_LINKED} sh /tests \
                 /var/opt/cladenc/harbor/tests"
            ),
        ]
    );
}

#[test]
fn the_verifier_runs_on_an_emptied_place_for_its_logs_which_are_then_copied_out() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.commands()[14..17],
        [
            format!(
                "docker exec --user 0 NAME sh -c {LOGS_LINKED} && {VERIFIER_EMPTIED} sh /logs \
                 /var/log/cladenc/harbor /logs/verifier 0"
            ),
            format!(
                "docker exec --env-file /dev/stdin --user 0 NAME sh -c {TIMER} /tests/test.sh 605"
            ),
            "docker cp NAME:/logs/verifier/. -".to_owned(),
        ]
    );
}

#[test]
fn the_logs_of_the_verifier_are_unpacked_in_the_logs_kept_with_the_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.docker.asked()[16].kept,
        tried.running().join("verifier/logs/verifier")
    );
}

#[test]
fn the_sandbox_of_a_trial_is_removed_once_its_verifier_has_run_and_nothing_more_is_asked() {
    let tried = trialled_by(Double::default());
    assert_eq!(tried.commands()[17..], [REMOVED]);
}

#[test]
fn a_trial_whose_verifier_writes_a_reward_of_one_completed_the_task() {
    let tried = trialled_by(Double::default().verifying(|_| true));
    assert_eq!(result_of(&tried), Some(TrialResult::Pass));
}

#[test]
fn a_trial_whose_verifier_writes_another_reward_failed() {
    let tried = trialled_by(Double::default().verifying(|_| false));
    assert_eq!(result_of(&tried), Some(TrialResult::Fail));
}

#[test]
fn a_trial_whose_verifier_writes_no_reward_is_inconclusive_for_no_verdict() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        result_of(&tried),
        Some(TrialResult::Inconclusive(Cause::NoVerdict))
    );
}

#[test]
fn the_reward_decides_whatever_status_the_verifier_exits_with() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&VERIFIER, &[Answer::Exited(1)]);
    assert_eq!(result_of(&trialled_by(docker)), Some(TrialResult::Pass));
}

#[test]
fn a_trial_whose_verifier_runs_past_its_time_limit_timed_out() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&VERIFIER, &[Answer::TimedOut]);
    assert_eq!(
        result_of(&trialled_by(docker)),
        Some(TrialResult::Inconclusive(Cause::TimedOut))
    );
}

#[test]
fn the_logs_of_the_verifier_are_copied_into_the_directory_of_the_running_trial() {
    let docker = Double::default().verifying(|trial| trial == "trial-0-benchmark+t1");
    assert_eq!(result_of(&trialled_by(docker)), Some(TrialResult::Pass));
}

#[test]
fn a_trial_whose_harness_exits_with_a_status_other_than_zero_is_verified_all_the_same() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&HARNESS, &[Answer::Exited(3)]);
    assert_eq!(result_of(&trialled_by(docker)), Some(TrialResult::Pass));
}

#[test]
fn the_reward_of_the_verifier_is_kept_with_the_trial() {
    let tried = trialled_by(Double::default().verifying(|_| true));
    assert_eq!(
        kept(&tried, "verifier/logs/verifier/reward.txt").as_deref(),
        Some("1")
    );
}

#[test]
fn the_verifier_of_a_holdout_task_decides_the_result_of_its_trial() {
    let (directory, outputs_directory) = given::made();
    let outputs_directory = outputs_directory.holding_out(&[TASK.to_owned()]);
    let docker = Double::default().verifying(|_| true).on(&directory);
    let environment = given::environment(TaskConfiguration::default());
    let trial = Trial {
        harness: 0,
        task: TASK,
        environment: &environment,
    };
    let trialled = sandboxes::run_trial(
        given::host(&docker, &outputs_directory),
        &LAUNCH,
        trial,
        &AtomicBool::new(false),
        &|_| (),
    );
    assert_eq!(
        trialled.ok().map(|trialled| trialled.result),
        Some(TrialResult::Pass)
    );
}

#[test]
fn the_result_of_a_trial_is_kept_with_it() {
    let tried = trialled_by(Double::default().verifying(|_| true));
    assert_eq!(kept(&tried, "result").as_deref(), Some("pass\n"));
}

#[test]
fn a_trial_that_has_ended_is_no_longer_among_the_running_ones() {
    let tried = trialled_by(Double::default());
    assert!(!tried.directory.exists(tried.running()));
}

#[test]
fn a_harness_that_ends_within_its_time_limit_ended() {
    let tried = trialled_by(Double::default());
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Ended));
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_zero_ended_all_the_same() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::Exited(1)]));
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Ended));
}

#[test]
fn a_harness_that_runs_past_its_time_limit_was_stopped() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Stopped));
}

#[test]
fn everything_in_the_sandbox_is_ended_once_the_harness_ran_past_its_time_limit() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(
        tried.commands()[8],
        "docker exec --user 0 NAME sh -c kill -9 -1"
    );
}

#[test]
fn a_trial_whose_harness_was_stopped_is_verified_all_the_same() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&HARNESS, &[Answer::TimedOut]);
    assert_eq!(result_of(&trialled_by(docker)), Some(TrialResult::Pass));
}

#[test]
fn that_the_harness_ended_is_kept_with_the_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(kept(&tried, "ending").as_deref(), Some("ended\n"));
}

#[test]
fn that_the_harness_was_stopped_is_kept_with_the_trial() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(kept(&tried, "ending").as_deref(), Some("stopped\n"));
}

#[test]
fn only_what_changed_while_the_harness_ran_is_kept_as_its_changes() {
    let listed = [
        Answer::Printed("C /app\n"),
        Answer::Printed("C /app\nA /app/new\n"),
    ];
    let tried = trialled_by(Double::default().answering(&["diff"], &listed));
    assert_eq!(kept(&tried, "changes").as_deref(), Some("A /app/new\n"));
}

#[test]
fn a_harness_that_changes_nothing_in_the_sandbox_has_no_change_kept() {
    let tried = trialled_by(Double::default());
    assert_eq!(kept(&tried, "changes").as_deref(), Some(""));
}

#[test]
fn the_harness_runs_as_the_user_the_task_names_for_it() {
    let configuration = TaskConfiguration {
        harness_user: Some("agent".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(with_a_user_of_the_image(), configuration);
    assert_eq!(given::following("--user", &tried.commands()[7]), ["agent"]);
}

#[test]
fn the_source_code_of_the_harness_is_owned_by_the_user_the_task_names_for_the_harness() {
    let configuration = TaskConfiguration {
        harness_user: Some("agent".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(
        tried.commands()[4],
        "docker exec --user 0 NAME chown -R agent /var/opt/cladenc/harness"
    );
}

#[test]
fn the_harness_runs_as_the_user_of_the_image_where_the_task_names_none() {
    let tried = trialled_by(with_a_user_of_the_image());
    assert_eq!(
        given::following("--user", &tried.commands()[7]),
        ["builder"]
    );
}

#[test]
fn the_verifier_runs_as_the_user_the_task_names_for_it() {
    let configuration = TaskConfiguration {
        verifier_user: Some("checker".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(with_a_user_of_the_image(), configuration);
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["checker"]
    );
}

#[test]
fn the_verifier_runs_as_the_user_of_the_image_where_the_task_names_none() {
    let tried = trialled_by(with_a_user_of_the_image());
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["builder"]
    );
}

#[test]
fn what_the_harness_left_in_the_working_directory_of_the_image_is_copied_out() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("\n/app\n")]);
    let tried = trialled_by(docker);
    assert_eq!(tried.commands()[9], "docker cp NAME:/app/. -");
}

#[test]
fn what_the_harness_left_in_its_working_directory_is_unpacked_in_the_work_of_the_trial() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("\n/app\n")]);
    let tried = trialled_by(docker);
    assert_eq!(tried.docker.asked()[9].kept, tried.running().join("work"));
}

#[test]
fn the_harness_is_told_where_it_is_what_the_task_asks_and_which_model_to_call() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.docker.asked()[7].variables,
        [
            ("CLADENC_HARNESS", "/var/opt/cladenc/harness"),
            ("CLADENC_INSTRUCTION", "Finish the work of t1.\n"),
            ("CLADENC_MODEL", test_inputs::MODEL),
            ("CLADENC_MODEL_URL", test_inputs::MODEL_URL),
        ]
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
    );
}

#[test]
fn the_harness_is_given_the_key_of_the_model_last_where_there_is_one() {
    let environment = given::environment(TaskConfiguration::default());
    let tried = given::trialled(Double::default(), &KEYED, &environment);
    assert_eq!(
        given::following("--env", &tried.commands()[7])
            .pop()
            .as_deref(),
        Some("CLADENC_MODEL_KEY")
    );
}

#[test]
fn the_variables_of_the_task_are_given_to_its_sandbox() {
    let configuration = TaskConfiguration {
        variables: vec![("MODE".to_owned(), "of the task".to_owned())],
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(tried.docker.asked()[0].input, "MODE=of the task\n");
}

#[test]
fn the_variables_the_task_sets_for_its_verifier_are_given_to_the_verifier_and_not_to_the_harness() {
    let configuration = TaskConfiguration {
        verifier_variables: vec![("MODE".to_owned(), "strict".to_owned())],
        ..TaskConfiguration::default()
    };
    let asked = trialled_with(Double::default(), configuration)
        .docker
        .asked();
    assert_eq!(
        [asked[7].input.as_str(), asked[15].input.as_str()],
        ["", "MODE=strict\n"]
    );
}

#[test]
fn the_harness_is_given_the_time_limit_the_task_sets_for_it() {
    let configuration = TaskConfiguration {
        harness_time_limit: Some(Duration::from_secs(60)),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[7]), Some("65"));
}

#[test]
fn the_sandbox_of_a_task_with_a_time_limit_stays_for_the_harness_the_verifier_and_the_copying() {
    let configuration = TaskConfiguration {
        harness_time_limit: Some(Duration::from_secs(60)),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[0]), Some("1260"));
}

#[test]
fn the_verifier_is_given_the_time_limit_the_task_sets_for_it() {
    let configuration = TaskConfiguration {
        verifier_time_limit: Duration::from_secs(120),
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[15]), Some("125"));
}

#[test]
fn the_time_limit_of_the_verifier_is_counted_from_the_moment_the_harness_ends() {
    let tried = trialled_by(taking(&HARNESS, 1100));
    assert_eq!(last_word_of(&tried.commands()[15]), Some("605"));
}

#[test]
fn the_seconds_of_a_trial_are_those_its_harness_ran() {
    let tried = trialled_by(taking(&HARNESS, 20));
    assert!(seconds_of(&tried).is_some_and(|seconds| seconds >= 0.02));
}

#[test]
fn the_seconds_of_a_trial_leave_out_those_its_verifier_ran() {
    let tried = trialled_by(taking(&VERIFIER, 500));
    assert!(seconds_of(&tried).is_some_and(|seconds| seconds < 0.5));
}

#[test]
fn the_seconds_of_a_trial_whose_harness_was_stopped_are_those_it_ran() {
    let docker = taking(&HARNESS, 20).answering(&HARNESS, &[Answer::TimedOut]);
    assert!(seconds_of(&trialled_by(docker)).is_some_and(|seconds| seconds >= 0.02));
}

#[test]
fn what_docker_says_when_it_brings_up_the_sandbox_is_kept_with_the_running_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(tried.docker.asked()[0].kept, tried.running().join("stderr"));
}

#[test]
fn what_the_harness_prints_is_kept_in_the_directory_of_the_running_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(tried.docker.asked()[7].kept, tried.running());
}

#[test]
fn what_the_verifier_prints_is_kept_in_the_directory_of_the_verifier_of_the_trial() {
    let tried = trialled_by(Double::default());
    assert_eq!(
        tried.docker.asked()[15].kept,
        tried.running().join("verifier")
    );
}

#[test]
fn a_collect_command_runs_after_the_harness_as_the_user_it_names_within_its_time_limit() {
    let tried = trialled_with(Double::default(), collecting(&[Some("collector")]));
    assert_eq!(
        tried.commands()[9],
        format!("docker exec --env-file /dev/stdin --user collector NAME sh -c {TIMER} ls /app 35")
    );
}

#[test]
fn a_collect_command_that_names_no_user_runs_as_the_user_of_the_image() {
    let tried = trialled_with(with_a_user_of_the_image(), collecting(&[None]));
    assert_eq!(
        given::following("--user", &tried.commands()[9]),
        ["builder"]
    );
}

#[test]
fn what_a_collect_command_prints_is_kept_in_a_directory_named_by_its_place_among_them() {
    let tried = trialled_with(Double::default(), collecting(&[None, None]));
    let asked = tried.docker.asked();
    assert_eq!(
        [&asked[9].kept, &asked[10].kept],
        [
            &tried.running().join("collection-hooks/1"),
            &tried.running().join("collection-hooks/2")
        ]
    );
}

#[test]
fn a_collect_command_that_exits_with_a_status_other_than_zero_does_not_end_the_trial() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&COLLECT, &[Answer::Exited(3)]);
    let tried = trialled_with(docker, collecting(&[None]));
    assert_eq!(result_of(&tried), Some(TrialResult::Pass));
}

#[test]
fn a_collect_command_that_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let docker = Double::default().answering(&COLLECT, &[Answer::Exited(125)]);
    let tried = trialled_with(docker, collecting(&[None]));
    let said = tried.running().join("collection-hooks/1/stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_trial_that_is_stopped_while_a_collect_command_runs_was_stopped() {
    let docker = Double::default().answering(&COLLECT, &[Answer::Stopped]);
    let tried = trialled_with(docker, collecting(&[None]));
    assert!(matches!(tried.result, Err(SandboxError::Stopped)));
}

#[test]
fn a_path_the_task_names_to_be_kept_is_copied_out_before_the_published_place_for_artifacts() {
    let configuration = TaskConfiguration {
        artifacts: vec![Artifact {
            source: "/app/out".to_owned(),
            destination: "out".to_owned(),
            exclude: Vec::new(),
        }],
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(
        tried.commands()[9..11],
        [
            "docker cp --follow-link NAME:/app/out -",
            "docker cp --follow-link NAME:/logs/artifacts -",
        ]
    );
}

#[test]
fn the_published_place_for_artifacts_that_the_task_names_to_be_kept_is_copied_out_once() {
    let configuration = TaskConfiguration {
        artifacts: vec![Artifact {
            source: "/logs/artifacts".to_owned(),
            destination: "published".to_owned(),
            exclude: Vec::new(),
        }],
        ..TaskConfiguration::default()
    };
    let tried = trialled_with(Double::default(), configuration);
    assert_eq!(
        tried
            .commands()
            .iter()
            .filter(|command| command.starts_with("docker cp --follow-link NAME:/logs/artifacts "))
            .count(),
        1
    );
}

#[test]
fn a_trial_whose_sandbox_docker_does_not_bring_up_names_the_file_that_holds_what_it_said() {
    let tried = trialled_by(Double::default().answering(&["--detach"], &[Answer::Exited(1)]));
    let said = tried.running().join("stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_trial_whose_harness_cannot_be_put_in_the_sandbox_is_not_carried_out() {
    let tried = trialled_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert!(matches!(tried.result, Err(SandboxError::Io(_))));
}

#[test]
fn the_sandbox_of_a_trial_that_is_not_carried_out_is_removed() {
    let tried = trialled_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert_eq!(tried.commands().pop().as_deref(), Some(REMOVED));
}

#[test]
fn a_trial_that_is_not_carried_out_stays_among_the_running_ones() {
    let tried = trialled_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert!(tried.directory.is_dir(tried.running()));
}

#[test]
fn a_trial_whose_place_for_artifacts_cannot_be_made_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", "/logs/artifacts"], &[REFUSED]);
    assert!(matches!(
        trialled_by(docker).result,
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn a_trial_whose_harness_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::Exited(125)]));
    let said = tried.running().join("stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_trial_that_is_stopped_while_its_harness_runs_was_stopped() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::Stopped]));
    assert!(matches!(tried.result, Err(SandboxError::Stopped)));
}

#[test]
fn a_trial_that_is_stopped_stays_among_the_running_ones() {
    let tried = trialled_by(Double::default().answering(&HARNESS, &[Answer::Stopped]));
    assert!(tried.directory.is_dir(tried.running()));
}

#[test]
fn a_trial_whose_tests_cannot_be_given_to_the_verifier_is_not_carried_out() {
    let owned = ["chown", "/var/opt/cladenc/harbor/tests"];
    let tried = trialled_by(Double::default().answering(&owned, &[REFUSED]));
    assert!(matches!(tried.result, Err(SandboxError::Io(_))));
}

#[test]
fn a_trial_whose_place_for_the_logs_of_the_verifier_cannot_be_emptied_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", "/logs/verifier"], &[REFUSED]);
    assert!(matches!(
        trialled_by(docker).result,
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn a_trial_whose_verifier_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let tried = trialled_by(Double::default().answering(&VERIFIER, &[Answer::Exited(125)]));
    let said = tried.running().join("verifier/stderr");
    assert!(matches!(&tried.result, Err(SandboxError::Docker(kept)) if *kept == said));
}

#[test]
fn a_trial_that_is_stopped_while_its_verifier_runs_was_stopped() {
    let tried = trialled_by(Double::default().answering(&VERIFIER, &[Answer::Stopped]));
    assert!(matches!(tried.result, Err(SandboxError::Stopped)));
}
