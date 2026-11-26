use std::fs;
use std::io;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

const KEEP_THE_TASKS_FOUND: &str = "if [ -d /var/log/cladenc/trials ]; then ls \
                                    /var/log/cladenc/trials/benchmark > tasks-found; fi";

const KEEP_WHAT_IS_GIVEN_TO_READ: &str = "if [ -d /var/log/cladenc/trials ]; then ls \
                                          /var/log/cladenc > given; fi";

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(
                &Disk,
                &test_inputs::benchmark(directory, &["t1", "t2", "t3", "t4", "t5"]),
            )
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        max_diversifications: 1,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 20.0,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t4".to_owned()],
        corroboration_tasks_percentage: 20.0,
        corroboration_tasks: vec!["benchmark/t4".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), KEEP_THE_TASKS_FOUND);
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    directory
}

static GIVEN_TO_THE_MAKERS_OF_TWO_CHILDREN: LazyLock<[String; 2]> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), KEEP_WHAT_IS_GIVEN_TO_READ);
    settings.max_diversifications = 2;
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    ["1", "2"].map(|child| {
        fs::read_to_string(
            outputs_directory
                .join("harnesses")
                .join(child)
                .join("given"),
        )
        .unwrap()
    })
});

#[test]
fn a_progenitor_finds_its_trials_on_the_tasks_that_are_not_held_out_and_no_other() {
    let directory = evolved();
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/1/tasks-found")).unwrap(),
        "t1\nt3\nt5\n"
    );
}

#[test]
fn the_primogenitor_that_is_cloned_for_the_first_time_is_given_its_trials_and_nothing_else() {
    assert_eq!(GIVEN_TO_THE_MAKERS_OF_TWO_CHILDREN[0], "trials\n");
}

#[test]
fn the_harness_that_is_cloned_next_is_given_the_child_it_made_or_the_line_it_descends_from() {
    assert!(
        ["children\ntrials\n", "line\ntrials\n"]
            .contains(&GIVEN_TO_THE_MAKERS_OF_TWO_CHILDREN[1].as_str())
    );
}
