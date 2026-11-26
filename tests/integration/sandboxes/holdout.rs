use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::genealogy::TrialResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Environment, Host, Launch, Program, SandboxError, Trial};
use tempfile::TempDir;

const SHOWN: &str = "benchmark/t1";
const HELD_OUT: &str = "benchmark/t2";
const LIST_THE_TRIALS: &str =
    "ls /var/log/cladenc/trials /var/log/cladenc/trials/benchmark > evidence";
const LOOK_FOR_THE_HOLDOUT_TASK: &str = "{ find /var/opt/cladenc /var/log/cladenc -name '*t2*'; \
                                         grep -r -l t2 /var/opt/cladenc /var/log/cladenc; } > \
                                         evidence; true";
const SAY_WHAT_IS_GIVEN: &str = "{ env | grep ^CLADENC_ | grep -v ^CLADENC_INSTRUCTION= | sort; \
                                 pwd; ls -R /var/opt/cladenc /var/log/cladenc 2>&1; }";
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

fn made() -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap()
    .holding_out(&[HELD_OUT.to_owned()]);
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    (directory, outputs_directory)
}

fn environment() -> Environment {
    Environment {
        image: test_inputs::IMAGE.to_owned(),
        ..Environment::default()
    }
}

fn trialled(
    run_command: &str,
    task: &str,
) -> (
    TempDir,
    OutputsDirectory,
    Result<sandboxes::Trialled, SandboxError>,
) {
    let (directory, outputs_directory) = made();
    let result = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch(run_command),
        Trial {
            harness: 0,
            task,
            environment: &environment(),
        },
        &AtomicBool::new(false),
        &|_| (),
    );
    (directory, outputs_directory, result)
}

fn given_in_the_trial_at(task: &str) -> String {
    let (_directory, outputs_directory, _result) = trialled(SAY_WHAT_IS_GIVEN, task);
    fs::read_to_string(outputs_directory.trial(0, task).join("stdout")).unwrap()
}

fn left_in_the_child_of_a_benchmarked_progenitor(run_command: &str) -> String {
    let (_directory, outputs_directory) = made();
    for task in [SHOWN, HELD_OUT] {
        outputs_directory.start_trial(0, task).unwrap();
        outputs_directory
            .end_trial(0, task, TrialResult::Fail)
            .unwrap();
    }
    let _ = sandboxes::run_diversification(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch(run_command),
        1,
        0,
        &AtomicBool::new(false),
    );
    fs::read_to_string(outputs_directory.harness(1).join("evidence")).unwrap()
}

#[test]
fn a_progenitor_is_given_its_trials_on_the_tasks_that_are_not_held_out_and_no_other() {
    assert_eq!(
        left_in_the_child_of_a_benchmarked_progenitor(LIST_THE_TRIALS),
        "/var/log/cladenc/trials:\nbenchmark\n\n/var/log/cladenc/trials/benchmark:\nt1\n"
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
fn the_harness_of_a_trial_on_a_holdout_task_is_given_what_that_of_any_trial_is() {
    assert_eq!(
        given_in_the_trial_at(HELD_OUT),
        given_in_the_trial_at(SHOWN)
    );
}
