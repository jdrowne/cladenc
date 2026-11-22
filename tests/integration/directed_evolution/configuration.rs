use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, ConfigurationError};
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const NEVER_SET: &str = "[verifier.env]\nKEY = \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET}\"\n";
const SAY_THE_MODE_OR_END_WITH_125: &str = "[ -e end-with-125 ] && exit 125; echo \"$MODE\"";

struct Evolved {
    directory: TempDir,
    found: Result<Found, DirectedEvolutionError>,
    said: String,
}

fn in_the_image(image: &str) -> String {
    format!("[environment]\ndocker_image = \"{image}\"\n")
}

fn settings(directory: &Path, task_tomls: &[(&str, &str)]) -> Settings {
    let benchmark = test_inputs::benchmark(directory, &["t1", "t2", "t3"]);
    for (task, task_toml) in task_tomls {
        fs::write(benchmark.join(task).join("task.toml"), task_toml).unwrap();
    }
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&benchmark).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: SAY_THE_MODE_OR_END_WITH_125.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 17.0,
        holdout_tasks: vec!["benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 17.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved(task_tomls: &[(&str, &str)]) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), task_tomls);
    let mut written = Vec::new();
    let found = directed_evolution::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut written,
    );
    Evolved {
        directory,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn with_t2_in(image: &str) -> Evolved {
    evolved(&[("t2", &in_the_image(image))])
}

fn stopped_and_resumed(task_toml_of_t2: &str, change: impl FnOnce(&Path)) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), &[("t2", task_toml_of_t2)]);
    let outputs_directory = directory.path().join("outputs");
    test_inputs::end_with_125_on(directory.path(), "t3");
    let _ = directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t3");
    change(&outputs_directory);
    let mut written = Vec::new();
    let found =
        directed_evolution::resume(&outputs_directory, &AtomicBool::new(false), &mut written);
    Evolved {
        directory,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn outputs_of(evolved: &Evolved) -> PathBuf {
    evolved.directory.path().join("outputs")
}

fn kept_task_toml_of_t2(outputs_directory: &Path) -> PathBuf {
    outputs_directory.join("benchmarks/benchmark/t2/task.toml")
}

fn printed_in_the_attempt_at(evolved: &Evolved, task: &str) -> String {
    let attempt = test_inputs::kept_attempt(&outputs_of(evolved), 0, task);
    fs::read_to_string(attempt.join("stdout")).unwrap()
}

#[test]
fn a_directed_evolution_builds_no_environment_for_a_task_whose_task_toml_names_an_image() {
    assert!(
        !with_t2_in(test_inputs::IMAGE)
            .said
            .contains("cladenc: building the environment of task benchmark/t2 ")
    );
}

#[test]
fn a_directed_evolution_says_that_it_takes_the_image_a_task_names_as_the_environment_of_the_task() {
    assert!(with_t2_in(test_inputs::IMAGE).said.contains(
        "cladenc: taking the image debian:stable-slim as the environment of task benchmark/t2, \
         as its task.toml says\n"
    ));
}

#[test]
fn a_directed_evolution_says_how_many_of_the_environments_it_built() {
    let evolved = with_t2_in(test_inputs::IMAGE);
    assert!(evolved.said.contains(&format!(
        "cladenc: built the environments of 2 of the 3 tasks: what Docker printed is kept in \
         {}/environments\n",
        outputs_of(&evolved).display()
    )));
}

#[test]
fn a_directed_evolution_in_which_every_task_names_an_image_says_of_none_that_it_was_built() {
    let image = in_the_image(test_inputs::IMAGE);
    let evolved = evolved(&[("t1", &image), ("t2", &image), ("t3", &image)]);
    assert!(!evolved.said.contains("cladenc: built the environment"));
}

#[test]
fn a_directed_evolution_in_which_every_task_names_an_image_runs_to_its_end() {
    let image = in_the_image(test_inputs::IMAGE);
    assert!(
        evolved(&[("t1", &image), ("t2", &image), ("t3", &image)])
            .found
            .is_ok()
    );
}

#[test]
fn an_attempt_at_a_task_that_names_an_image_is_carried_out_in_a_container_of_that_image() {
    let evolved = with_t2_in(test_inputs::IMAGE);
    let attempt = test_inputs::kept_attempt(&outputs_of(&evolved), 0, "benchmark/t2");
    assert!(!attempt.join("work").exists());
}

#[test]
fn an_attempt_at_a_task_is_given_the_variables_its_task_toml_names() {
    let evolved = evolved(&[("t1", "[environment.env]\nMODE = \"strict\"\n")]);
    assert_eq!(
        printed_in_the_attempt_at(&evolved, "benchmark/t1"),
        "strict\n"
    );
}

#[test]
fn an_attempt_at_a_task_is_given_no_variable_that_another_task_names() {
    let evolved = evolved(&[("t1", "[environment.env]\nMODE = \"strict\"\n")]);
    assert_eq!(printed_in_the_attempt_at(&evolved, "benchmark/t2"), "\n");
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_stops_the_directed_evolution_naming_both() {
    assert!(matches!(
        with_t2_in(NOT_ON_THE_MACHINE).found,
        Err(DirectedEvolutionError::NotOnTheMachine { task, image, .. })
            if task == "benchmark/t2" && image == NOT_ON_THE_MACHINE
    ));
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_leaves_the_directed_evolution_no_journal() {
    let evolved = with_t2_in(NOT_ON_THE_MACHINE);
    assert!(!journal::path(&outputs_of(&evolved)).exists());
}

#[test]
fn a_task_whose_image_is_not_on_the_machine_leaves_the_environments_after_it_unbuilt() {
    assert!(
        !with_t2_in(NOT_ON_THE_MACHINE)
            .said
            .contains("environment of task benchmark/t3")
    );
}

#[test]
fn a_resumed_directed_evolution_takes_the_image_a_task_names_again() {
    let image = in_the_image(test_inputs::IMAGE);
    assert!(stopped_and_resumed(&image, |_| ()).said.contains(
        "cladenc: taking the image debian:stable-slim as the environment of task benchmark/t2, \
         as its task.toml says\n"
    ));
}

#[test]
fn a_resumed_directed_evolution_whose_task_names_an_image_runs_to_its_end() {
    let image = in_the_image(test_inputs::IMAGE);
    assert!(stopped_and_resumed(&image, |_| ()).found.is_ok());
}

#[test]
fn a_resumed_directed_evolution_reads_the_task_toml_of_the_copy_of_a_task() {
    let evolved = stopped_and_resumed("", |outputs_directory| {
        fs::write(kept_task_toml_of_t2(outputs_directory), NEVER_SET).unwrap();
    });
    assert!(matches!(
        evolved.found,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::NotSet { .. },
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_a_task_whose_copy_no_longer_holds_a_task_toml() {
    let evolved = stopped_and_resumed("", |outputs_directory| {
        fs::remove_file(kept_task_toml_of_t2(outputs_directory)).unwrap();
    });
    assert!(matches!(
        evolved.found,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::Missing,
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_a_task_whose_task_toml_is_no_longer_valid() {
    let evolved = stopped_and_resumed("", |outputs_directory| {
        fs::write(
            kept_task_toml_of_t2(outputs_directory),
            "[agent]\nnetwork_mode = \"private\"\n",
        )
        .unwrap();
    });
    assert!(matches!(
        evolved.found,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::NotValid { line: 2, .. },
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_an_image_that_is_no_longer_on_the_machine() {
    let evolved = stopped_and_resumed("", |outputs_directory| {
        fs::write(
            kept_task_toml_of_t2(outputs_directory),
            in_the_image(NOT_ON_THE_MACHINE),
        )
        .unwrap();
    });
    assert!(matches!(
        evolved.found,
        Err(DirectedEvolutionError::NotOnTheMachine { task, .. }) if task == "benchmark/t2"
    ));
}

fn with_a_time_limit_of_2_seconds_for_every_harness() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let within_2 = "[agent]\ntimeout_sec = 2\n";
    let settings = Settings {
        harness_run_command: "[ -d /var/log/cladenc/attempts ] || sleep 300".to_owned(),
        ..settings(
            directory.path(),
            &[("t1", within_2), ("t2", within_2), ("t3", within_2)],
        )
    };
    let mut written = Vec::new();
    let found = directed_evolution::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut written,
    );
    Evolved {
        directory,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn with_an_environment_of_t2_that_builds_for_too_long() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(
        directory.path(),
        &[("t2", "[environment]\nbuild_timeout_sec = 2\n")],
    );
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN sleep 300\n", test_inputs::IMAGE),
    )
    .unwrap();
    let mut written = Vec::new();
    let found = directed_evolution::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut written,
    );
    Evolved {
        directory,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

#[test]
fn a_harness_that_runs_past_the_time_limit_of_its_task_is_said_to_have_been_stopped_at_it() {
    assert!(
        with_a_time_limit_of_2_seconds_for_every_harness()
            .said
            .contains(
                "cladenc: harness 0 did not complete task benchmark/t1, after it was stopped at \
                 its time limit: what the attempt left is kept in "
            )
    );
}

#[test]
fn a_directed_evolution_whose_harnesses_all_run_out_of_time_runs_to_its_end() {
    assert!(
        with_a_time_limit_of_2_seconds_for_every_harness()
            .found
            .is_ok()
    );
}

#[test]
fn an_environment_whose_build_runs_past_the_time_limit_of_its_task_stops_the_directed_evolution() {
    assert!(matches!(
        with_an_environment_of_t2_that_builds_for_too_long().found,
        Err(DirectedEvolutionError::Environment {
            task,
            error: ContainerError::TimeLimit,
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn an_environment_whose_build_runs_past_its_time_limit_is_said_to_have_run_past_it() {
    assert!(
        with_an_environment_of_t2_that_builds_for_too_long()
            .said
            .contains(
                "cladenc: the build of the environment of task benchmark/t2 ran past its time \
                 limit: what Docker printed is kept in "
            )
    );
}

#[test]
fn a_task_that_asks_for_more_cpus_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved(&[("t2", "[environment]\ncpus = 100000\n")]).found,
        Err(DirectedEvolutionError::TooLarge { task, key: "cpus", .. }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_task_that_asks_for_more_memory_than_the_machine_has_stops_the_directed_evolution() {
    assert!(matches!(
        evolved(&[("t2", "[environment]\nmemory_mb = 100000000\n")]).found,
        Err(DirectedEvolutionError::TooLarge { task, key: "memory_mb", .. })
            if task == "benchmark/t2"
    ));
}

#[test]
fn a_task_that_asks_for_more_than_the_machine_has_leaves_the_directed_evolution_no_journal() {
    let evolved = evolved(&[("t2", "[environment]\ncpus = 100000\n")]);
    assert!(!journal::path(&outputs_of(&evolved)).exists());
}

#[test]
fn a_directed_evolution_whose_tasks_ask_for_what_the_machine_has_runs_to_its_end() {
    let within = "[environment]\ncpus = 1\nmemory_mb = 512\n";
    assert!(
        evolved(&[("t1", within), ("t2", within), ("t3", within)])
            .found
            .is_ok()
    );
}
