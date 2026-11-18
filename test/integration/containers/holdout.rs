use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, ContainerError, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const SHOWN: &str = "benchmark/t1";
const HELD_OUT: &str = "benchmark/t2";
const LIST_THE_ATTEMPTS: &str =
    "ls /var/log/cladenc/attempts /var/log/cladenc/attempts/benchmark > evidence";
const LOOK_FOR_THE_HOLDOUT_TASK: &str = "{ find /var/lib/cladenc /var/log/cladenc -name '*t2*'; \
                                         grep -r -l t2 /var/lib/cladenc /var/log/cladenc; } > \
                                         evidence; true";
const SAY_WHAT_IS_GIVEN: &str = "{ env | grep ^CLADENC_ | grep -v ^CLADENC_INSTRUCTION= | sort; \
                                 pwd; ls -R /var/lib/cladenc /var/log/cladenc 2>&1; } > given";

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
        procreation_timeout: 30,
        attempt_timeout: 30,
    }
}

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory =
        OutputsDirectory::create(&directory.path().join("outputs"), &primogenitor)
            .unwrap()
            .holding_out(&[HELD_OUT.to_owned()]);
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn attempted(
    run_command: &str,
    task: &str,
) -> (
    TempDir,
    OutputsDirectory,
    Result<AttemptResult, ContainerError>,
) {
    let (directory, outputs_directory) = made();
    let result = containers::run_attempt(
        &outputs_directory,
        &launch(run_command),
        0,
        task,
        &AtomicBool::new(false),
    );
    (directory, outputs_directory, result)
}

fn given_in_the_attempt_at(task: &str) -> String {
    let (_directory, outputs_directory, _result) = attempted(SAY_WHAT_IS_GIVEN, task);
    fs::read_to_string(outputs_directory.attempt(0, task).join("work/given")).unwrap()
}

fn left_in_the_child_of_a_benchmarked_progenitor(run_command: &str) -> String {
    let (_directory, outputs_directory) = made();
    for task in [SHOWN, HELD_OUT] {
        outputs_directory.start_attempt(0, task).unwrap();
        outputs_directory
            .end_attempt(0, task, AttemptResult::NotCompleted)
            .unwrap();
    }
    let _ = containers::run_procreation(
        &outputs_directory,
        &launch(run_command),
        1,
        0,
        &AtomicBool::new(false),
    );
    fs::read_to_string(outputs_directory.harness(1).join("evidence")).unwrap()
}

#[test]
fn a_progenitor_is_given_its_attempts_at_the_tasks_that_are_not_held_out_and_no_other() {
    assert_eq!(
        left_in_the_child_of_a_benchmarked_progenitor(LIST_THE_ATTEMPTS),
        "/var/log/cladenc/attempts:\nbenchmark\n\n/var/log/cladenc/attempts/benchmark:\nt1\n"
    );
}

#[test]
fn nothing_a_progenitor_is_given_is_named_after_a_holdout_task_or_holds_its_id() {
    assert_eq!(
        left_in_the_child_of_a_benchmarked_progenitor(LOOK_FOR_THE_HOLDOUT_TASK),
        ""
    );
}

#[test]
fn the_harness_of_an_attempt_at_a_holdout_task_is_given_what_that_of_any_attempt_is() {
    assert_eq!(
        given_in_the_attempt_at(HELD_OUT),
        given_in_the_attempt_at(SHOWN)
    );
}

#[test]
fn the_grader_of_a_holdout_task_says_whether_the_harness_completed_it() {
    let (_directory, _outputs_directory, result) = attempted("echo t2 > solved", HELD_OUT);
    assert!(matches!(result, Ok(AttemptResult::Completed)));
}

#[test]
fn an_attempt_at_a_holdout_task_is_kept_under_holdout_with_what_it_left() {
    let (_directory, outputs_directory, _result) = attempted("echo t2 > solved", HELD_OUT);
    assert_eq!(
        fs::read_to_string(
            outputs_directory
                .path()
                .join("holdout/0/benchmark/t2/work/solved")
        )
        .unwrap(),
        "t2\n"
    );
}

#[test]
fn an_attempt_at_a_holdout_task_leaves_nothing_under_attempts() {
    let (_directory, outputs_directory, _result) = attempted("echo t2 > solved", HELD_OUT);
    assert_eq!(
        fs::read_dir(outputs_directory.attempts(0)).unwrap().count(),
        0
    );
}
