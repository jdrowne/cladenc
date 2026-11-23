use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::benchmark::{Artifact, EnvironmentConfiguration, TaskConfiguration};
use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{
    self, EMPTIED, HEALTH_CHECK, LAUNCH, LOGS_LINKED, TASK, TESTS_LINKED, TIMER, Tried,
    VERIFIER_EMPTIED,
};
use crate::containers::{Checked, ContainerError, Environment, GraderImage, Launch, Retry};
use crate::genealogy::AttemptResult;

const RETRIED: Launch<'static> = Launch {
    max_grader_health_check_retries: 1,
    ..LAUNCH
};

const GRADER_IMAGE: &str = "grader:t1";
const COPIED: [&str; 2] = ["cp", "--follow-link"];
const CHECK: [&str; 2] = ["exec", HEALTH_CHECK];
const REFUSED: Answer = Answer::Refused("docker is not there");
const REMOVED: &str = "docker rm --force NAME";
const ROOM_MADE: &str = r#"rm -rf "$1" && mkdir -p "${1%/*}/""#;
const GIVEN: &str = "OUTPUTS/running/attempt-0-benchmark+t1/given-to-the-grader";
const BROUGHT_UP: &str = "docker run --rm --pull never --name NAME --label \
                          cladenc.directed-evolution=OUTPUTS --add-host \
                          host.docker.internal:host-gateway";

fn apart(holds_grader: bool, configuration: TaskConfiguration) -> Environment {
    Environment {
        grader: Some(GraderImage {
            name: GRADER_IMAGE.to_owned(),
            holds_grader,
        }),
        ..given::environment(configuration)
    }
}

fn graded_apart(docker: Double) -> Tried {
    graded_with(docker, TaskConfiguration::default())
}

fn graded_with(docker: Double, configuration: TaskConfiguration) -> Tried {
    given::attempted(docker, &LAUNCH, &apart(true, configuration))
}

fn result_of(tried: &Tried) -> Option<AttemptResult> {
    tried.result.as_ref().ok().map(|attempted| attempted.result)
}

fn leaving_a_directory() -> Double {
    Double::default().doing(&COPIED, |asked| {
        let left = asked
            .arguments
            .last()
            .map(PathBuf::from)
            .unwrap_or_default();
        fs::create_dir(&left).unwrap();
        fs::write(left.join("made.txt"), "made\n").unwrap();
    })
}

fn leaving_a_file() -> Double {
    Double::default().doing(&COPIED, |asked| {
        let left = asked
            .arguments
            .last()
            .map(PathBuf::from)
            .unwrap_or_default();
        fs::write(left, "made\n").unwrap();
    })
}

fn with_an_environment_of_the_grader() -> TaskConfiguration {
    TaskConfiguration {
        grader_environment: Some(EnvironmentConfiguration {
            working_directory: Some("/grading".to_owned()),
            variables: vec![("MODE".to_owned(), "of the grader".to_owned())],
            cpus: Some(0.5),
            memory_megabytes: Some(512),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    }
}

fn with_a_checked_grader() -> TaskConfiguration {
    TaskConfiguration {
        grader_environment: Some(EnvironmentConfiguration {
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
fn the_container_of_the_task_is_removed_before_the_container_of_its_grader_is_brought_up() {
    let tried = graded_apart(Double::default());
    assert_eq!(
        tried.commands()[10..12],
        [
            REMOVED.to_owned(),
            format!("{BROUGHT_UP} --env-file /dev/stdin --detach grader:t1 sleep 1200"),
        ]
    );
}

#[test]
fn the_logs_that_harbor_names_are_linked_in_the_container_of_the_grader() {
    let tried = graded_apart(Double::default());
    assert_eq!(
        tried.commands()[13],
        format!("docker exec --user 0 NAME sh -c {LOGS_LINKED} sh /logs /var/log/cladenc/harbor")
    );
}

#[test]
fn a_grader_that_its_image_holds_is_given_no_tests_and_runs_in_its_own_container() {
    let tried = graded_apart(Double::default());
    assert_eq!(
        tried.commands()[14..],
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
            REMOVED.to_owned(),
        ]
    );
}

#[test]
fn a_grader_that_its_image_does_not_hold_is_given_the_tests_of_the_task() {
    let environment = apart(false, TaskConfiguration::default());
    let tried = given::attempted(Double::default(), &LAUNCH, &environment);
    assert_eq!(
        tried.commands()[14..18],
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
fn a_grader_in_a_container_of_its_own_decides_the_result_of_the_attempt() {
    let tried = graded_apart(Double::default().grading(|_| true));
    assert_eq!(result_of(&tried), Some(AttemptResult::Completed));
}

#[test]
fn what_docker_says_when_it_brings_up_the_container_of_a_grader_is_kept_with_the_grader() {
    let tried = graded_apart(Double::default());
    assert_eq!(
        tried.docker.asked()[11].kept,
        tried.running().join("grader/stderr")
    );
}

#[test]
fn the_container_of_a_grader_is_given_the_settings_of_the_environment_of_the_grader() {
    let tried = graded_with(Double::default(), with_an_environment_of_the_grader());
    assert_eq!(
        tried.commands()[11],
        format!(
            "{BROUGHT_UP} --workdir /grading --cpus 0.5 --memory 512m --env-file /dev/stdin \
             --detach grader:t1 sleep 1200"
        )
    );
}

#[test]
fn the_container_of_a_grader_is_given_the_variables_of_the_environment_of_the_grader() {
    let tried = graded_with(Double::default(), with_an_environment_of_the_grader());
    assert_eq!(tried.docker.asked()[11].input, "MODE=of the grader\n");
}

#[test]
fn a_directory_the_harness_left_as_artifacts_is_given_to_the_grader_at_the_same_path() {
    let tried = graded_apart(leaving_a_directory());
    assert_eq!(
        tried.commands()[14..17],
        [
            format!("docker exec --user 0 NAME sh -c {EMPTIED} sh /logs/artifacts"),
            format!("docker cp {GIVEN}/. NAME:/logs/artifacts"),
            "docker exec --user 0 NAME chown -R 0 /logs/artifacts".to_owned(),
        ]
    );
}

#[test]
fn a_file_the_harness_left_as_artifacts_is_given_to_the_grader_at_the_same_path() {
    let tried = graded_apart(leaving_a_file());
    assert_eq!(
        tried.commands()[14..17],
        [
            format!("docker exec --user 0 NAME sh -c {ROOM_MADE} sh /logs/artifacts"),
            format!("docker cp {GIVEN} NAME:/logs/artifacts"),
            "docker exec --user 0 NAME chown -R 0 /logs/artifacts".to_owned(),
        ]
    );
}

#[test]
fn a_path_the_task_names_to_be_kept_is_given_to_the_grader_before_the_published_place() {
    let configuration = TaskConfiguration {
        artifacts: vec![Artifact {
            source: "/app/out".to_owned(),
            destination: "out".to_owned(),
            exclude: Vec::new(),
        }],
        ..TaskConfiguration::default()
    };
    let tried = graded_with(leaving_a_directory(), configuration);
    assert_eq!(
        tried.commands()[16],
        format!("docker cp {GIVEN}/. NAME:/app/out")
    );
}

#[test]
fn what_is_given_to_the_grader_is_owned_by_the_user_the_task_names_for_the_grader() {
    let configuration = TaskConfiguration {
        grader_user: Some("judge".to_owned()),
        ..TaskConfiguration::default()
    };
    let tried = graded_with(leaving_a_directory(), configuration);
    assert_eq!(
        tried.commands()[16],
        "docker exec --user 0 NAME chown -R judge /logs/artifacts"
    );
}

#[test]
fn the_copy_that_is_given_to_the_grader_is_not_kept_with_the_attempt() {
    let tried = graded_apart(leaving_a_directory());
    let attempt = tried.outputs_directory.attempt(0, TASK);
    assert!(!attempt.join("given-to-the-grader").exists());
}

#[test]
fn a_grader_in_a_container_of_its_own_runs_as_the_user_of_its_own_image() {
    let users = [
        Answer::Printed("task\n/\n"),
        Answer::Printed("checker\n/\n"),
    ];
    let tried = graded_apart(Double::default().answering(&["inspect"], &users));
    assert_eq!(
        given::following("--user", &tried.commands()[15]),
        ["checker"]
    );
}

#[test]
fn the_health_check_of_a_grader_runs_in_its_container_before_anything_else() {
    let tried = graded_with(Double::default(), with_a_checked_grader());
    assert_eq!(
        tried.commands()[13],
        format!("docker exec --env-file /dev/stdin --user 0 NAME sh -c {TIMER} {HEALTH_CHECK} 35")
    );
}

#[test]
fn what_the_health_check_of_a_grader_prints_is_kept_with_the_grader() {
    let tried = graded_with(Double::default(), with_a_checked_grader());
    assert_eq!(
        tried.docker.asked()[13].kept,
        tried.running().join("grader/health-check")
    );
}

#[test]
fn the_container_of_a_grader_with_a_health_check_stays_for_the_longest_the_check_can_take() {
    let tried = graded_with(Double::default(), with_a_checked_grader());
    assert_eq!(tried.commands()[11].rsplit(' ').next(), Some("1260"));
}

#[test]
fn a_grader_whose_health_check_does_not_pass_is_not_run() {
    let tried = graded_with(never_healthy(), with_a_checked_grader());
    let printed = tried.running().join("grader/health-check");
    assert!(matches!(
        &tried.result,
        Err(ContainerError::GraderHealthCheck { tries: 1, kept }) if *kept == printed
    ));
}

#[test]
fn the_container_of_a_grader_whose_health_check_does_not_pass_is_removed() {
    let tried = graded_with(never_healthy(), with_a_checked_grader());
    assert_eq!(tried.commands()[14..], [REMOVED]);
}

#[test]
fn a_grader_whose_health_check_does_not_pass_is_brought_up_again_as_often_as_is_allowed() {
    let environment = apart(true, with_a_checked_grader());
    let tried = given::attempted(never_healthy(), &RETRIED, &environment);
    let brought_up = tried
        .commands()
        .iter()
        .filter(|command| command.contains("--detach grader:t1"))
        .count();
    assert_eq!(brought_up, 2);
}

#[test]
fn each_time_a_grader_is_brought_up_again_for_its_health_check_is_told() {
    let environment = apart(true, with_a_checked_grader());
    let tried = given::attempted(never_healthy(), &RETRIED, &environment);
    assert_eq!(
        tried.retries,
        [Retry {
            checked: Checked::Grader,
            number: 1,
            most: 1,
            wait: Duration::ZERO,
        }]
    );
}

#[test]
fn a_grader_whose_health_check_never_passes_names_how_often_it_was_tried() {
    let environment = apart(true, with_a_checked_grader());
    let tried = given::attempted(never_healthy(), &RETRIED, &environment);
    assert!(matches!(
        tried.result,
        Err(ContainerError::GraderHealthCheck { tries: 2, .. })
    ));
}

#[test]
fn a_grader_whose_health_check_passes_once_it_is_brought_up_again_is_run() {
    let docker = Double::default()
        .grading(|_| true)
        .answering(&CHECK, &[Answer::Exited(1), Answer::Exited(0)]);
    let environment = apart(true, with_a_checked_grader());
    let tried = given::attempted(docker, &RETRIED, &environment);
    assert_eq!(result_of(&tried), Some(AttemptResult::Completed));
}

#[test]
fn a_grader_whose_container_docker_does_not_bring_up_names_the_file_that_holds_what_it_said() {
    let detached = [Answer::Exited(0), Answer::Exited(1)];
    let tried = graded_apart(Double::default().answering(&["--detach"], &detached));
    let said = tried.running().join("grader/stderr");
    assert!(matches!(&tried.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn an_attempt_whose_logs_cannot_be_linked_in_the_container_of_its_grader_is_not_carried_out() {
    let docker = Double::default().answering(&["exec", LOGS_LINKED], &[REFUSED]);
    assert!(matches!(
        graded_apart(docker).result,
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn an_attempt_whose_artifacts_cannot_be_given_to_its_grader_is_not_carried_out() {
    let docker = leaving_a_directory().answering(&["chown", "/logs/artifacts"], &[REFUSED]);
    assert!(matches!(
        graded_apart(docker).result,
        Err(ContainerError::Io(_))
    ));
}
