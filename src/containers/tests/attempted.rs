use std::time::Duration;

use crate::benchmark::{Artifact, CollectCommand, TaskConfiguration};
use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{
    self, ARTIFACTS_MADE, EMPTIED, LAUNCH, LOGS_LINKED, TASK, TESTS_LINKED, TIMER, Tried,
    VERIFIER_EMPTIED,
};
use crate::containers::{ContainerError, HarnessEnding, Launch};
use crate::genealogy::AttemptResult;
use crate::model::Model;

const KEYED: Launch<'static> = Launch {
    model: Model {
        key: Some(test_inputs::MODEL_KEY),
        ..LAUNCH.model
    },
    ..LAUNCH
};

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const GRADER: [&str; 2] = ["exec", "/tests/test.sh"];
const COLLECT: [&str; 2] = ["exec", "ls /app"];
const REFUSED: Answer = Answer::Refused("docker is not there");
const REMOVED: &str = "docker rm --force NAME";
const TOLD: &str = "--env CLADENC_HARNESS --env CLADENC_INSTRUCTION --env CLADENC_MODEL \
                    --env CLADENC_MODEL_URL";

fn attempted_by(docker: Double) -> Tried {
    attempted_with(docker, TaskConfiguration::default())
}

fn attempted_with(docker: Double, configuration: TaskConfiguration) -> Tried {
    given::attempted(docker, &LAUNCH, &given::environment(configuration))
}

fn result_of(tried: &Tried) -> Option<AttemptResult> {
    tried.result.as_ref().ok().map(|attempted| attempted.result)
}

fn ending_of(tried: &Tried) -> Option<HarnessEnding> {
    tried.result.as_ref().ok().map(|attempted| attempted.ending)
}

fn kept(tried: &Tried, file: &str) -> Option<String> {
    tried
        .directory
        .read_to_string(tried.outputs_directory.attempt(0, TASK).join(file))
        .ok()
}

fn last_word_of(command: &str) -> Option<&str> {
    command.rsplit(' ').next()
}

fn with_a_user_of_the_image() -> Double {
    Double::default().answering(&["inspect"], &[Answer::Printed("builder\n/\n")])
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
fn an_attempt_brings_up_a_container_of_the_task_that_stays_past_the_harness_and_the_grader() {
    let tried = attempted_by(Double::default());
    assert_eq!(
        tried.commands()[0],
        "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
         --add-host host.docker.internal:host-gateway --env-file /dev/stdin --detach \
         debian:stable-slim sleep 4294968495"
    );
}

#[test]
fn the_source_code_of_the_harness_is_put_in_the_container_and_owned_by_who_runs_it() {
    let tried = attempted_by(Double::default());
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
    let tried = attempted_by(Double::default());
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
    let tried = attempted_by(Double::default());
    assert_eq!(
        tried.commands()[7],
        format!(
            "docker exec --env-file /dev/stdin {TOLD} --user 0 NAME sh -c {TIMER} run a harness \
             4294967300"
        )
    );
}

#[test]
fn what_changed_in_the_container_is_asked_for_before_and_after_the_harness_runs() {
    let commands = attempted_by(Double::default()).commands();
    assert_eq!(
        [commands[6].as_str(), commands[8].as_str()],
        ["docker diff NAME"; 2]
    );
}

#[test]
fn what_the_harness_left_in_the_published_place_for_artifacts_is_copied_out() {
    let tried = attempted_by(Double::default());
    assert_eq!(
        tried.commands()[9],
        "docker cp --follow-link NAME:/logs/artifacts \
         OUTPUTS/running/attempt-0-benchmark+t1/artifacts/logs/artifacts"
    );
}

#[test]
fn the_tests_of_the_task_are_put_in_the_container_only_once_the_harness_has_ended() {
    let tried = attempted_by(Double::default());
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
fn the_grader_runs_on_an_emptied_place_for_its_logs_which_are_then_copied_out() {
    let tried = attempted_by(Double::default());
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
            "docker cp NAME:/logs/verifier/. \
             OUTPUTS/running/attempt-0-benchmark+t1/grader/logs/verifier"
                .to_owned(),
        ]
    );
}

#[test]
fn the_container_of_an_attempt_is_removed_once_its_grader_has_run_and_nothing_more_is_asked() {
    let tried = attempted_by(Double::default());
    assert_eq!(tried.commands()[17..], [REMOVED]);
}

#[test]
fn an_attempt_whose_grader_writes_a_reward_of_one_completed_the_task() {
    let tried = attempted_by(Double::default().grading(|_| true));
    assert_eq!(result_of(&tried), Some(AttemptResult::Completed));
}

#[test]
fn an_attempt_whose_grader_writes_another_reward_did_not_complete_the_task() {
    let tried = attempted_by(Double::default().grading(|_| false));
    assert_eq!(result_of(&tried), Some(AttemptResult::NotCompleted));
}

#[test]
fn an_attempt_whose_grader_writes_no_reward_did_not_complete_the_task() {
    let tried = attempted_by(Double::default());
    assert_eq!(result_of(&tried), Some(AttemptResult::NotCompleted));
}

#[test]
fn the_reward_decides_whatever_status_the_grader_exits_with() {
    let docker = Double::default()
        .grading(|_| true)
        .answering(&GRADER, &[Answer::Exited(1)]);
    assert_eq!(
        result_of(&attempted_by(docker)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn an_attempt_whose_grader_runs_past_its_time_limit_timed_out() {
    let docker = Double::default()
        .grading(|_| true)
        .answering(&GRADER, &[Answer::TimedOut]);
    assert_eq!(
        result_of(&attempted_by(docker)),
        Some(AttemptResult::TimedOut)
    );
}

#[test]
fn the_logs_of_the_grader_are_copied_into_the_directory_of_the_running_attempt() {
    let docker = Double::default().grading(|attempt| attempt == "attempt-0-benchmark+t1");
    assert_eq!(
        result_of(&attempted_by(docker)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn the_result_of_an_attempt_is_kept_with_it() {
    let tried = attempted_by(Double::default().grading(|_| true));
    assert_eq!(kept(&tried, "result").as_deref(), Some("completed\n"));
}

#[test]
fn an_attempt_that_has_ended_is_no_longer_among_the_running_ones() {
    let tried = attempted_by(Double::default());
    assert!(!tried.directory.exists(tried.running()));
}

#[test]
fn a_harness_that_ends_within_its_time_limit_ended() {
    let tried = attempted_by(Double::default());
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Ended));
}

#[test]
fn a_harness_that_exits_with_a_status_other_than_zero_ended_all_the_same() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::Exited(1)]));
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Ended));
}

#[test]
fn a_harness_that_runs_past_its_time_limit_was_stopped() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(ending_of(&tried), Some(HarnessEnding::Stopped));
}

#[test]
fn everything_in_the_container_is_ended_once_the_harness_ran_past_its_time_limit() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(
        tried.commands()[8],
        "docker exec --user 0 NAME sh -c kill -9 -1"
    );
}

#[test]
fn an_attempt_whose_harness_was_stopped_is_graded_all_the_same() {
    let docker = Double::default()
        .grading(|_| true)
        .answering(&HARNESS, &[Answer::TimedOut]);
    assert_eq!(
        result_of(&attempted_by(docker)),
        Some(AttemptResult::Completed)
    );
}

#[test]
fn that_the_harness_ended_is_kept_with_the_attempt() {
    let tried = attempted_by(Double::default());
    assert_eq!(kept(&tried, "ending").as_deref(), Some("ended\n"));
}

#[test]
fn that_the_harness_was_stopped_is_kept_with_the_attempt() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::TimedOut]));
    assert_eq!(kept(&tried, "ending").as_deref(), Some("stopped\n"));
}

#[test]
fn only_what_changed_while_the_harness_ran_is_kept_as_its_changes() {
    let listed = [
        Answer::Printed("C /app\n"),
        Answer::Printed("C /app\nA /app/new\n"),
    ];
    let tried = attempted_by(Double::default().answering(&["diff"], &listed));
    assert_eq!(kept(&tried, "changes").as_deref(), Some("A /app/new\n"));
}

#[test]
fn a_harness_that_changes_nothing_in_the_container_has_no_change_kept() {
    let tried = attempted_by(Double::default());
    assert_eq!(kept(&tried, "changes").as_deref(), Some(""));
}

#[test]
fn the_harness_runs_as_the_user_the_task_names_for_it() {
    let configuration = TaskConfiguration {
        harness_user: Some("agent".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(with_a_user_of_the_image(), configuration);
    assert_eq!(given::following("--user", &tried.commands()[7]), ["agent"]);
}

#[test]
fn the_source_code_of_the_harness_is_owned_by_the_user_the_task_names_for_the_harness() {
    let configuration = TaskConfiguration {
        harness_user: Some("agent".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(
        tried.commands()[4],
        "docker exec --user 0 NAME chown -R agent /var/opt/cladenc/harness"
    );
}

#[test]
fn the_harness_runs_as_the_user_of_the_image_where_the_task_names_none() {
    let tried = attempted_by(with_a_user_of_the_image());
    assert_eq!(
        given::following("--user", &tried.commands()[7]),
        ["builder"]
    );
}

#[test]
fn the_grader_runs_as_the_user_the_task_names_for_it() {
    let configuration = TaskConfiguration {
        grader_user: Some("checker".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(with_a_user_of_the_image(), configuration);
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["checker"]
    );
}

#[test]
fn the_grader_runs_as_the_user_of_the_image_where_the_task_names_none() {
    let tried = attempted_by(with_a_user_of_the_image());
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["builder"]
    );
}

#[test]
fn what_the_harness_left_in_the_working_directory_of_the_image_is_copied_out() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("\n/app\n")]);
    let tried = attempted_by(docker);
    assert_eq!(
        tried.commands()[9],
        "docker cp NAME:/app/. OUTPUTS/running/attempt-0-benchmark+t1/work"
    );
}

#[test]
fn the_harness_is_told_where_it_is_what_the_task_asks_and_which_model_to_call() {
    let tried = attempted_by(Double::default());
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
    let tried = given::attempted(Double::default(), &KEYED, &environment);
    assert_eq!(
        given::following("--env", &tried.commands()[7])
            .pop()
            .as_deref(),
        Some("CLADENC_MODEL_KEY")
    );
}

#[test]
fn the_variables_of_the_task_are_given_to_its_container() {
    let configuration = TaskConfiguration {
        variables: vec![("MODE".to_owned(), "of the task".to_owned())],
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(tried.docker.asked()[0].input, "MODE=of the task\n");
}

#[test]
fn the_variables_the_task_sets_for_its_grader_are_given_to_the_grader_and_not_to_the_harness() {
    let configuration = TaskConfiguration {
        grader_variables: vec![("MODE".to_owned(), "strict".to_owned())],
        ..TaskConfiguration::default()
    };
    let asked = attempted_with(Double::default(), configuration)
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
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[7]), Some("65"));
}

#[test]
fn the_container_of_a_task_with_a_time_limit_stays_for_the_harness_the_grader_and_the_copying() {
    let configuration = TaskConfiguration {
        harness_time_limit: Some(Duration::from_secs(60)),
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[0]), Some("1260"));
}

#[test]
fn the_grader_is_given_the_time_limit_the_task_sets_for_it() {
    let configuration = TaskConfiguration {
        grader_time_limit: Duration::from_secs(120),
        ..TaskConfiguration::default()
    };
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(last_word_of(&tried.commands()[15]), Some("125"));
}

#[test]
fn what_docker_says_when_it_brings_up_the_container_is_kept_with_the_running_attempt() {
    let tried = attempted_by(Double::default());
    assert_eq!(tried.docker.asked()[0].kept, tried.running().join("stderr"));
}

#[test]
fn what_the_harness_prints_is_kept_in_the_directory_of_the_running_attempt() {
    let tried = attempted_by(Double::default());
    assert_eq!(tried.docker.asked()[7].kept, tried.running());
}

#[test]
fn what_the_grader_prints_is_kept_in_the_directory_of_the_grader_of_the_attempt() {
    let tried = attempted_by(Double::default());
    assert_eq!(
        tried.docker.asked()[15].kept,
        tried.running().join("grader")
    );
}

#[test]
fn a_collect_command_runs_after_the_harness_as_the_user_it_names_within_its_time_limit() {
    let tried = attempted_with(Double::default(), collecting(&[Some("collector")]));
    assert_eq!(
        tried.commands()[9],
        format!("docker exec --env-file /dev/stdin --user collector NAME sh -c {TIMER} ls /app 35")
    );
}

#[test]
fn a_collect_command_that_names_no_user_runs_as_the_user_of_the_image() {
    let tried = attempted_with(with_a_user_of_the_image(), collecting(&[None]));
    assert_eq!(
        given::following("--user", &tried.commands()[9]),
        ["builder"]
    );
}

#[test]
fn what_a_collect_command_prints_is_kept_in_a_directory_named_by_its_place_among_them() {
    let tried = attempted_with(Double::default(), collecting(&[None, None]));
    let asked = tried.docker.asked();
    assert_eq!(
        [&asked[9].kept, &asked[10].kept],
        [
            &tried.running().join("collect/1"),
            &tried.running().join("collect/2")
        ]
    );
}

#[test]
fn a_collect_command_that_exits_with_a_status_other_than_zero_does_not_end_the_attempt() {
    let docker = Double::default().answering(&COLLECT, &[Answer::Exited(3)]);
    let tried = attempted_with(docker, collecting(&[None]));
    assert_eq!(result_of(&tried), Some(AttemptResult::NotCompleted));
}

#[test]
fn a_collect_command_that_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let docker = Double::default().answering(&COLLECT, &[Answer::Exited(125)]);
    let tried = attempted_with(docker, collecting(&[None]));
    let said = tried.running().join("collect/1/stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn an_attempt_that_is_stopped_while_a_collect_command_runs_was_stopped() {
    let docker = Double::default().answering(&COLLECT, &[Answer::Stopped]);
    let tried = attempted_with(docker, collecting(&[None]));
    assert!(matches!(tried.result, Err(ContainerError::Stopped)));
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
    let tried = attempted_with(Double::default(), configuration);
    assert_eq!(
        tried.commands()[9..11],
        [
            "docker cp --follow-link NAME:/app/out \
             OUTPUTS/running/attempt-0-benchmark+t1/artifacts/out",
            "docker cp --follow-link NAME:/logs/artifacts \
             OUTPUTS/running/attempt-0-benchmark+t1/artifacts/logs/artifacts",
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
    let tried = attempted_with(Double::default(), configuration);
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
fn an_attempt_whose_container_docker_does_not_bring_up_names_the_file_that_holds_what_it_said() {
    let tried = attempted_by(Double::default().answering(&["--detach"], &[Answer::Exited(1)]));
    let said = tried.running().join("stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn an_attempt_whose_harness_cannot_be_put_in_the_container_is_not_carried_out() {
    let tried = attempted_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert!(matches!(tried.result, Err(ContainerError::Io(_))));
}

#[test]
fn the_container_of_an_attempt_that_is_not_carried_out_is_removed() {
    let tried = attempted_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert_eq!(tried.commands().pop().as_deref(), Some(REMOVED));
}

#[test]
fn an_attempt_that_is_not_carried_out_stays_among_the_running_ones() {
    let tried = attempted_by(Double::default().answering(&["cp"], &[REFUSED]));
    assert!(tried.directory.is_dir(tried.running()));
}

#[test]
fn an_attempt_whose_place_for_artifacts_cannot_be_made_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", "/logs/artifacts"], &[REFUSED]);
    assert!(matches!(
        attempted_by(docker).result,
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn an_attempt_whose_harness_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::Exited(125)]));
    let said = tried.running().join("stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn an_attempt_that_is_stopped_while_its_harness_runs_was_stopped() {
    let tried = attempted_by(Double::default().answering(&HARNESS, &[Answer::Stopped]));
    assert!(matches!(tried.result, Err(ContainerError::Stopped)));
}

#[test]
fn an_attempt_whose_tests_cannot_be_given_to_the_grader_is_not_carried_out() {
    let owned = ["chown", "/var/opt/cladenc/harbor/tests"];
    let tried = attempted_by(Double::default().answering(&owned, &[REFUSED]));
    assert!(matches!(tried.result, Err(ContainerError::Io(_))));
}

#[test]
fn an_attempt_whose_place_for_the_logs_of_the_grader_cannot_be_emptied_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", "/logs/verifier"], &[REFUSED]);
    assert!(matches!(
        attempted_by(docker).result,
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn an_attempt_whose_grader_docker_does_not_run_names_the_file_that_holds_what_docker_said() {
    let tried = attempted_by(Double::default().answering(&GRADER, &[Answer::Exited(125)]));
    let said = tried.running().join("grader/stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn an_attempt_that_is_stopped_while_its_grader_runs_was_stopped() {
    let tried = attempted_by(Double::default().answering(&GRADER, &[Answer::Stopped]));
    assert!(matches!(tried.result, Err(ContainerError::Stopped)));
}
