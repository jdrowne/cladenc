use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Creation, Host, Launch, Program};
use cladenc::file_system::Disk;
use cladenc::genealogy::{CreationOutcome, Experiment, TrialResult};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const EXPERIMENT: Experiment = Experiment {
    child: 3,
    progenitor: 1,
};

fn launch<'a>(image: &'a str, run_command: &'a str, build_command: Option<&'a str>) -> Launch<'a> {
    Launch {
        image,
        run_command,
        build_command,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    }
}

fn trialled(outputs_directory: &OutputsDirectory, harness: u64) {
    for task in ["benchmark/t1", "benchmark/t2"] {
        outputs_directory.start_trial(harness, task).unwrap();
        outputs_directory
            .end_trial(harness, task, TrialResult::NotCompleted)
            .unwrap();
        fs::write(
            outputs_directory.trial(harness, task).join("stdout"),
            format!("trial of {harness}\n"),
        )
        .unwrap();
    }
}

fn diversified(
    outputs_directory: &OutputsDirectory,
    child: u64,
    progenitor: u64,
    outcome: CreationOutcome,
) {
    outputs_directory
        .start_diversification(child, progenitor)
        .unwrap();
    fs::write(
        outputs_directory
            .running_diversification(child)
            .join("child/name"),
        format!("{child}\n"),
    )
    .unwrap();
    outputs_directory
        .end_diversification(child, progenitor, outcome)
        .unwrap();
}

fn family() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap()
    .holding_out(&["benchmark/t2".to_owned()]);
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    trialled(&outputs_directory, 0);
    diversified(&outputs_directory, 1, 0, CreationOutcome::Created);
    trialled(&outputs_directory, 1);
    diversified(&outputs_directory, 2, 1, CreationOutcome::Created);
    diversified(&outputs_directory, 3, 1, CreationOutcome::Created);
    diversified(&outputs_directory, 4, 1, CreationOutcome::Failed);
    (directory, outputs_directory)
}

type Established = (TempDir, OutputsDirectory, Result<Creation, ContainerError>);

fn established_in(image: &str, run_command: &str, build_command: Option<&str>) -> Established {
    let (directory, outputs_directory) = family();
    let result = containers::run_baseline(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch(image, run_command, build_command),
        5,
        EXPERIMENT,
        &AtomicBool::new(false),
    );
    (directory, outputs_directory, result)
}

fn established(run_command: &str) -> Established {
    established_in(test_inputs::IMAGE, run_command, None)
}

fn left(run_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = established(run_command);
    fs::read_to_string(outputs_directory.harness(5).join(file)).unwrap()
}

fn found(look: &str) -> String {
    left(&format!("{look} > evidence"), "evidence")
}

#[test]
fn the_source_code_that_runs_for_a_baseline_is_a_copy_of_that_of_the_primogenitor() {
    assert_eq!(
        found("ls \"$CLADENC_HARNESS\""),
        "harness.sh\nprompt.txt\nprompts\n"
    );
}

#[test]
fn the_source_code_that_is_diversified_for_a_baseline_is_a_clone_of_the_progenitor() {
    assert_eq!(found("cat name"), "1\n");
}

#[test]
fn the_working_directory_of_a_baseline_is_the_clone_that_is_diversified() {
    assert_eq!(found("pwd"), "/var/opt/cladenc/work\n");
}

#[test]
fn the_container_of_a_baseline_holds_nothing_under_var_lib_that_is_named_after_cladenc() {
    assert_eq!(
        found("{ ls -d /var/lib/cladenc 2> /dev/null; echo looked; }"),
        "looked\n"
    );
}

#[test]
fn what_the_primogenitor_leaves_is_kept_as_the_singleton_control_group_harness() {
    assert_eq!(left("echo changed > name", "name"), "changed\n");
}

#[test]
fn the_primogenitor_is_given_the_diversification_prompt_the_progenitor_is_given() {
    assert_eq!(
        left("printf %s \"$CLADENC_INSTRUCTION\" > given", "given"),
        containers::DIVERSIFICATION_PROMPT
    );
}

#[test]
fn the_primogenitor_is_given_the_trials_of_the_progenitor_to_read() {
    assert_eq!(
        found("cat /var/log/cladenc/trials/benchmark/t1/stdout"),
        "trial of 1\n"
    );
}

#[test]
fn the_primogenitor_is_given_no_trial_of_the_progenitor_at_a_holdout_task() {
    assert_eq!(found("ls /var/log/cladenc/trials/benchmark"), "t1\n");
}

#[test]
fn the_primogenitor_is_given_the_harnesses_the_progenitor_descends_from() {
    assert_eq!(found("ls /var/log/cladenc/line"), "0\n");
}

#[test]
fn the_primogenitor_is_given_the_children_the_progenitor_made_before_and_no_later_one() {
    assert_eq!(found("ls /var/log/cladenc/children"), "2\n");
}

#[test]
fn a_singleton_control_group_harness_is_built_where_a_build_command_is_given() {
    let (_directory, outputs_directory, _result) = established_in(
        test_inputs::IMAGE,
        "touch changed",
        Some("echo built > built"),
    );
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(5).join("built")).unwrap(),
        "built\n"
    );
}
