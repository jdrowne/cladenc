use std::path::PathBuf;

use crate::benchmark::{Artifact, TaskConfiguration};
use crate::genealogy::TrialResult;
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{self, LAUNCH, TASK, Tried};

const HARNESS: [&str; 2] = ["exec", "run a harness"];
const COPIED_OUT: [&str; 2] = ["cp", "--follow-link"];
const REFUSED: Answer = Answer::Refused("no such path in the container");
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
        let to = asked
            .arguments
            .last()
            .map(PathBuf::from)
            .unwrap_or_default();
        if asked_for {
            directory.create_dir_all(&to).unwrap();
            for file in files {
                directory.write(to.join(file), "").unwrap();
            }
        }
    })
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
    let commands = trialled(Double::default(), configuration).commands();
    assert!(
        commands.contains(
            &"docker cp --follow-link NAME:/logs/artifacts \
          OUTPUTS/running/trial-0-benchmark+t1/artifacts/published"
                .to_owned()
        )
    );
}

#[test]
fn a_trial_goes_on_where_a_path_the_task_names_cannot_be_copied_out() {
    let docker = Double::default()
        .verifying(|_| true)
        .answering(&COPIED_OUT, &[REFUSED]);
    let tried = trialled(docker, naming(&[NEVER_MADE], &[]));
    assert_eq!(
        tried.result.ok().map(|trialled| trialled.result),
        Some(TrialResult::Pass)
    );
}

#[test]
fn a_path_named_after_one_that_cannot_be_copied_out_is_kept() {
    let docker =
        leaving(Double::default(), OUT, &["a.txt"]).answering(&COPIED_OUT, &[REFUSED, COPIED]);
    let tried = trialled(docker, naming(&[NEVER_MADE, OUT], &[]));
    assert_eq!(kept_of(&tried, "app"), ["out"]);
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
