use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use cladenc::dataset::{Dataset, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Environment, Host, Launch, Program};
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "dataset/t1";
const NEVER_READY: &str = "[environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";

struct Prepared {
    _directory: TempDir,
    _built: Image,
    outputs_directory: OutputsDirectory,
    environment: Environment,
}

fn prepared(task_toml: &str) -> Prepared {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let dataset = test_inputs::dataset(directory.path(), &["t1"]);
    fs::write(dataset.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_dataset(&Dataset::read(&Disk, &dataset).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", FINISHED);
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let built = test_inputs::image();
    sandboxes::build_environment(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        TASK,
        &built.name,
        configuration.build_time_limit,
        &AtomicBool::new(false),
    )
    .unwrap();
    let environment = Environment {
        image: built.name.clone(),
        configuration,
        verifier: None,
    };
    Prepared {
        _directory: directory,
        _built: built,
        outputs_directory,
        environment,
    }
}

fn sandboxes_of(outputs_directory: &OutputsDirectory) -> usize {
    let label = format!(
        "label=cladenc.directed-evolution={}",
        fs::canonicalize(outputs_directory.path())
            .unwrap()
            .display()
    );
    let listed = Command::new("docker")
        .args(["ps", "--quiet", "--filter", &label])
        .output()
        .unwrap();
    String::from_utf8(listed.stdout).unwrap().lines().count()
}

fn sandboxes_at_each_retry(task_toml: &str, max_retries: u64) -> Vec<usize> {
    let prepared = prepared(task_toml);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: FINISH,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: max_retries,
    };
    let counted = RefCell::new(Vec::new());
    let _ = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &prepared.outputs_directory,
        },
        &launch,
        sandboxes::Trial {
            harness: 0,
            task: TASK,
            environment: &prepared.environment,
        },
        &AtomicBool::new(false),
        &|_| {
            counted
                .borrow_mut()
                .push(sandboxes_of(&prepared.outputs_directory));
        },
    );
    counted.into_inner()
}

#[test]
fn the_sandbox_of_a_task_is_removed_before_the_wait_for_a_retry() {
    assert_eq!(sandboxes_at_each_retry(NEVER_READY, 2), [0, 0]);
}
