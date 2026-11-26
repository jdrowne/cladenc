use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::genealogy::{CreationOutcome, TrialResult};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Host, Launch, Program};
use tempfile::TempDir;

const LOOK_FOR_THE_HOLDOUT_TASK: &str = "{ find /var/opt/cladenc /var/log/cladenc -name '*t2*'; \
                                         grep -r -l t2 /var/opt/cladenc /var/log/cladenc; } > \
                                         evidence; true";

fn launch(run_command: &str) -> Launch<'_> {
    Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
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
            .join("stdout"),
        format!("making {child}\n"),
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
    let build = outputs_directory.start_build(1).unwrap();
    fs::write(build.join("stdout"), "building\n").unwrap();
    trialled(&outputs_directory, 1);
    diversified(&outputs_directory, 2, 0, CreationOutcome::Failed);
    diversified(&outputs_directory, 3, 1, CreationOutcome::Created);
    trialled(&outputs_directory, 3);
    (directory, outputs_directory)
}

fn made_by(progenitor: u64, run_command: &str) -> (TempDir, OutputsDirectory) {
    let (directory, outputs_directory) = family();
    let _ = sandboxes::run_diversification(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch(run_command),
        4,
        progenitor,
        &AtomicBool::new(false),
    );
    (directory, outputs_directory)
}

fn found_by(progenitor: u64, look: &str) -> String {
    let (_directory, outputs_directory) = made_by(progenitor, &format!("{look} > evidence"));
    fs::read_to_string(outputs_directory.harness(4).join("evidence")).unwrap()
}

#[test]
fn a_progenitor_is_given_a_directory_for_each_harness_it_descends_from() {
    assert_eq!(found_by(3, "ls /var/log/cladenc/line"), "0\n1\n");
}

#[test]
fn a_progenitor_that_has_made_no_child_is_given_no_directory_of_children() {
    assert_eq!(found_by(3, "ls /var/log/cladenc"), "line\ntrials\n");
}

#[test]
fn the_primogenitor_is_given_the_children_it_made_created_or_not_and_no_line() {
    assert_eq!(
        found_by(0, "{ ls /var/log/cladenc; ls /var/log/cladenc/children; }"),
        "children\ntrials\n1\n2\n"
    );
}

#[test]
fn a_progenitor_is_given_no_harness_that_is_neither_of_its_line_nor_its_child() {
    assert_eq!(
        found_by(
            1,
            "{ ls /var/log/cladenc/line; ls /var/log/cladenc/children; }"
        ),
        "0\n3\n"
    );
}

#[test]
fn of_a_harness_that_was_made_and_built_a_progenitor_is_given_all_four_directories() {
    assert_eq!(
        found_by(3, "ls /var/log/cladenc/line/1"),
        "build\ndiversification\nharness\ntrials\n"
    );
}

#[test]
fn of_the_primogenitor_a_progenitor_is_given_its_source_code_and_its_trials_alone() {
    assert_eq!(
        found_by(3, "ls /var/log/cladenc/line/0"),
        "harness\ntrials\n"
    );
}

#[test]
fn of_a_child_that_was_not_created_a_progenitor_is_given_no_trials() {
    assert_eq!(
        found_by(0, "ls /var/log/cladenc/children/2"),
        "diversification\nharness\n"
    );
}

#[test]
fn a_progenitor_reads_which_harness_made_a_child_and_how_the_diversification_ended() {
    assert_eq!(
        found_by(
            0,
            "cat /var/log/cladenc/children/2/diversification/progenitor \
             /var/log/cladenc/children/2/diversification/outcome"
        ),
        "0\nfailed\n"
    );
}

#[test]
fn a_progenitor_reads_what_the_harness_that_made_one_of_its_line_printed() {
    assert_eq!(
        found_by(3, "cat /var/log/cladenc/line/1/diversification/stdout"),
        "making 1\n"
    );
}

#[test]
fn a_progenitor_reads_what_the_build_of_a_harness_of_its_line_printed() {
    assert_eq!(
        found_by(3, "cat /var/log/cladenc/line/1/build/stdout"),
        "building\n"
    );
}

#[test]
fn a_progenitor_reads_the_source_code_of_a_harness_of_its_line() {
    assert_eq!(
        found_by(3, "cat /var/log/cladenc/line/0/harness/harness.sh"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_progenitor_reads_the_trials_of_a_harness_of_its_line_at_the_tasks_that_are_not_held_out() {
    assert_eq!(
        found_by(3, "ls /var/log/cladenc/line/1/trials/benchmark"),
        "t1\n"
    );
}

#[test]
fn a_progenitor_reads_the_trials_of_a_child_it_made_at_the_tasks_that_are_not_held_out() {
    assert_eq!(
        found_by(0, "ls /var/log/cladenc/children/1/trials/benchmark"),
        "t1\n"
    );
}

#[test]
fn nothing_given_of_the_line_or_of_the_children_is_named_after_a_holdout_task_or_holds_its_id() {
    let (_directory, outputs_directory) = made_by(1, LOOK_FOR_THE_HOLDOUT_TASK);
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(4).join("evidence")).unwrap(),
        ""
    );
}

#[test]
fn a_progenitor_cannot_change_what_it_is_given_of_a_harness_of_its_line() {
    let (_directory, outputs_directory) =
        made_by(3, "touch /var/log/cladenc/line/0/harness/written");
    assert!(!outputs_directory.harness(0).join("written").exists());
}
