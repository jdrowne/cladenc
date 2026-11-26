use std::path::PathBuf;

use crate::benchmark::{Artifact, TaskConfiguration};
use crate::genealogy::TrialResult;
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{self, LAUNCH, TASK, Tried};
use crate::sandboxes::{Copied, SandboxError};

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const COPIED_OUT: [&str; 2] = ["cp", "--follow-link"];
const REFUSED: Answer = Answer::Refused("no such path in the container");
const NOT_COPIED: Answer = Answer::Refused("no space left on device");
const ASKED: [&str; 2] = ["exec", r#"[ -e "$1" ] || echo none"#];
const NONE: Answer = Answer::Printed("none\n");
const WHETHER_OUT_IS_THERE: &str =
    r#"docker exec --workdir / --user 0 NAME sh -c [ -e "$1" ] || echo none sh /app/out"#;
const COPIED: Answer = Answer::Printed("");
const OUT: &str = "/app/out";
const NEVER_MADE: &str = "/app/never-made";
const PUBLISHED: &str = "/logs/artifacts";

fn naming(sources: &[&str], exclude: &[&str]) -> TaskConfiguration {
    TaskConfiguration {
        artifacts: sources
            .iter()
            .map(|source| Artifact {
                source: (*source).to_owned(),
                destination: source.trim_start_matches('/').to_owned(),
                exclude: exclude
                    .iter()
                    .map(|pattern| (*pattern).to_owned())
                    .collect(),
            })
            .collect(),
        ..TaskConfiguration::default()
    }
}

fn leaving(docker: Double, at: &'static str, files: &'static [&'static str]) -> Double {
    docker.doing(&COPIED_OUT, move |directory, asked| {
        let from = format!(":{at}");
        let asked_for = asked
            .arguments
            .iter()
            .any(|argument| argument.ends_with(&from));
        let to = asked.kept.join("left");
        if asked_for {
            directory.create_dir_all(&to).unwrap();
            for file in files {
                directory.write(to.join(file), "").unwrap();
            }
        }
    })
}

fn not_in_the_sandbox(docker: Double, copied: &[Answer]) -> Double {
    docker
        .answering(&COPIED_OUT, copied)
        .answering(&ASKED, &[NONE])
}

fn not_copying() -> Double {
    Double::default().answering(&COPIED_OUT, &[NOT_COPIED])
}

fn copies_asked_of(tried: &Tried) -> usize {
    tried
        .commands()
        .iter()
        .filter(|command| command.contains(" cp --follow-link "))
        .count()
}

fn trialled(docker: Double, configuration: TaskConfiguration) -> Tried {
    given::trialled(docker, &LAUNCH, &given::environment(configuration))
}

fn artifacts_of(tried: &Tried) -> PathBuf {
    tried.outputs_directory.trial(0, TASK).join("artifacts")
}

fn kept_of(tried: &Tried, directory: &str) -> Vec<String> {
    tried
        .directory
        .names_in(artifacts_of(tried).join(directory))
}

#[test]
fn what_the_patterns_of_a_path_match_is_left_out_of_what_is_kept_of_it() {
    let docker = leaving(Double::default(), OUT, &["a.txt", "draft.tmp"]);
    let tried = trialled(docker, naming(&[OUT], &["*.tmp"]));
    assert_eq!(kept_of(&tried, "app/out"), ["a.txt"]);
}

#[test]
fn the_paths_a_task_names_are_copied_out_before_its_verifier_runs() {
    let commands = trialled(Double::default(), naming(&[OUT], &[])).commands();
    let copied = commands
        .iter()
        .rposition(|command| command.contains(" cp --follow-link "));
    let verified = commands
        .iter()
        .position(|command| command.contains(" /tests/test.sh "));
    assert!(matches!((copied, verified), (Some(copied), Some(verified)) if copied < verified));
}

#[test]
fn the_published_place_that_a_task_names_is_kept_at_the_destination_the_task_gives() {
    let configuration = TaskConfiguration {
        artifacts: vec![Artifact {
            source: PUBLISHED.to_owned(),
            destination: "published".to_owned(),
            exclude: Vec::new(),
        }],
        ..TaskConfiguration::default()
    };
    let docker = leaving(Double::default(), PUBLISHED, &["a.txt"]);
    let tried = trialled(docker, configuration);
    assert_eq!(kept_of(&tried, "published"), ["a.txt"]);
}

#[test]
fn a_trial_goes_on_where_a_path_the_task_names_is_not_in_the_sandbox() {
    let docker = not_in_the_sandbox(Double::default().verifying(|_| true), &[REFUSED]);
    let tried = trialled(docker, naming(&[NEVER_MADE], &[]));
    assert_eq!(
        tried.result.ok().map(|trialled| trialled.result),
        Some(TrialResult::Pass)
    );
}

#[test]
fn a_path_named_after_one_that_is_not_in_the_sandbox_is_kept() {
    let docker = not_in_the_sandbox(
        leaving(Double::default(), OUT, &["a.txt"]),
        &[REFUSED, COPIED],
    );
    let tried = trialled(docker, naming(&[NEVER_MADE, OUT], &[]));
    assert_eq!(kept_of(&tried, "app"), ["out"]);
}

#[test]
fn the_sandbox_is_asked_whether_anything_is_at_a_path_that_was_not_copied_out() {
    let tried = trialled(not_copying(), naming(&[OUT], &[]));
    assert!(
        tried
            .commands()
            .iter()
            .any(|command| command == WHETHER_OUT_IS_THERE)
    );
}

#[test]
fn a_trial_whose_harness_left_a_path_that_cannot_be_copied_out_is_given_no_result() {
    let tried = trialled(not_copying(), naming(&[OUT], &[]));
    assert!(matches!(
        &tried.result,
        Err(SandboxError::NotCopiedOut { left: Copied::Artifact(path), .. }) if path == OUT
    ));
}

#[test]
fn what_docker_said_of_the_copy_is_the_reason_a_path_was_not_copied_out() {
    let tried = trialled(not_copying(), naming(&[OUT], &[]));
    assert!(matches!(
        &tried.result,
        Err(SandboxError::NotCopiedOut { error, .. })
            if error.to_string() == "no space left on device"
    ));
}

#[test]
fn no_path_is_copied_out_after_one_that_could_not_be() {
    let tried = trialled(not_copying(), naming(&[OUT, NEVER_MADE], &[]));
    assert_eq!(copies_asked_of(&tried), 1);
}

#[test]
fn the_verifier_of_a_trial_whose_harness_left_a_path_that_was_not_copied_out_is_not_run() {
    let tried = trialled(not_copying(), naming(&[OUT], &[]));
    assert!(
        !tried
            .commands()
            .iter()
            .any(|command| command.contains(" /tests/test.sh "))
    );
}

#[test]
fn a_trial_whose_harness_left_a_path_that_was_not_copied_out_keeps_no_result() {
    let tried = trialled(not_copying(), naming(&[OUT], &[]));
    assert!(
        !tried
            .directory
            .exists(tried.outputs_directory.trial(0, TASK))
    );
}

#[test]
fn a_trial_is_given_no_result_where_logs_artifacts_cannot_be_copied_out() {
    let tried = trialled(not_copying(), TaskConfiguration::default());
    assert!(matches!(
        &tried.result,
        Err(SandboxError::NotCopiedOut { left: Copied::Artifact(path), .. }) if path == PUBLISHED
    ));
}

#[test]
fn a_trial_whose_harness_leaves_nothing_to_keep_has_no_directory_of_artifacts() {
    let docker = leaving(Double::default(), PUBLISHED, &[]);
    let tried = trialled(docker, TaskConfiguration::default());
    assert!(!tried.directory.exists(artifacts_of(&tried)));
}

#[test]
fn what_a_harness_that_was_stopped_left_at_a_path_the_task_names_is_kept() {
    let docker =
        leaving(Double::default(), OUT, &["a.txt"]).answering(&HARNESS, &[Answer::TimedOut]);
    let tried = trialled(docker, naming(&[OUT], &[]));
    assert_eq!(kept_of(&tried, "app/out"), ["a.txt"]);
}
