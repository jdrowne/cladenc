use std::path::Path;

use crate::benchmark::ConfigurationError;
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;
use crate::sandboxes::tests::double::{Answer, Double};

const A_TRIAL: [&str; 2] = ["run a harness", "trial-0-benchmark+t2"];
const IMAGE_GIVEN: &str = "[environment]\ndocker_image = \"an-image:given\"\n";
const NEVER_SET: &str = "[verifier.env]\nKEY = \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET}\"\n";
const NOT_VALID: &str = "[agent]\nnetwork_mode = \"private\"\n";
const REFUSED: Answer = Answer::Refused("docker is not there");
const ENVIRONMENT_OF_T1_BUILT: &str = "docker build --tag cladenc-environments-0123456789abcdef:0 \
                                       --label cladenc.task=benchmark/t1 --label \
                                       cladenc.directed-evolution=OUTPUTS \
                                       OUTPUTS/benchmarks/benchmark/t1/environment";

fn stopping() -> Double {
    Double::default().answering(&A_TRIAL, &[Answer::Exited(125)])
}

fn stopped() -> Evolved {
    given::started(stopping(), given::unchanged)
}

fn with_an_image_of_t1(directory: &Memory, _: &mut Settings) {
    given::configured(directory, "t1", IMAGE_GIVEN);
}

fn resumed_with_an_image_of_t1() -> Evolved {
    let stopped = given::started(stopping(), with_an_image_of_t1);
    given::resumed(stopped, Double::default())
}

fn resumed_with_the_copy_of_t1(change: impl FnOnce(&Memory, &Path), docker: Double) -> Evolved {
    let stopped = stopped();
    let kept = stopped.outputs().join("benchmarks/benchmark/t1/task.toml");
    change(stopped.files(), &kept);
    given::resumed(stopped, docker)
}

fn resumed_with_the_copy_of_t1_holding(task_toml: &str) -> Evolved {
    resumed_with_the_copy_of_t1(
        |files, kept| files.write(kept, task_toml).unwrap(),
        Double::default(),
    )
}

#[test]
fn a_resumed_directed_evolution_takes_the_image_a_task_names_again() {
    assert!(resumed_with_an_image_of_t1().said.contains(
        "cladenc: taking the image an-image:given as the environment of task benchmark/t1, as \
         its task.toml says\n"
    ));
}

#[test]
fn a_resumed_directed_evolution_whose_task_names_an_image_runs_to_its_end() {
    assert!(resumed_with_an_image_of_t1().result.is_ok());
}

#[test]
fn a_resumed_directed_evolution_reads_the_task_toml_of_the_copy_of_a_task() {
    assert!(matches!(
        resumed_with_the_copy_of_t1_holding(NEVER_SET).result,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::NotSet { .. },
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_a_task_whose_copy_no_longer_holds_a_task_toml() {
    let resumed = resumed_with_the_copy_of_t1(
        |files, kept| files.remove_file(kept).unwrap(),
        Double::default(),
    );
    assert!(matches!(
        resumed.result,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::Missing,
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_a_task_whose_task_toml_is_no_longer_valid() {
    assert!(matches!(
        resumed_with_the_copy_of_t1_holding(NOT_VALID).result,
        Err(DirectedEvolutionError::Configuration {
            task,
            error: ConfigurationError::NotValid { line: 2, .. },
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_resumed_directed_evolution_stops_for_an_image_that_is_no_longer_on_the_machine() {
    let resumed = resumed_with_the_copy_of_t1(
        |files, kept| files.write(kept, IMAGE_GIVEN).unwrap(),
        Double::default().answering(&["image", "inspect"], &[REFUSED]),
    );
    assert!(matches!(
        resumed.result,
        Err(DirectedEvolutionError::NotOnTheMachine { task, .. }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_resumed_directed_evolution_says_that_it_builds_each_environment_after_it_says_how_far_it_got()
{
    let resumed = given::resumed(stopped(), Double::default());
    assert_eq!(
        resumed.lines()[1..3],
        [
            "cladenc: building the environment of task benchmark/t1 as the image \
             cladenc-environments-0123456789abcdef:0",
            "cladenc: building the environment of task benchmark/t2 as the image \
             cladenc-environments-0123456789abcdef:1",
        ]
    );
}

#[test]
fn a_resumed_directed_evolution_builds_the_environments_from_the_tasks_in_its_outputs_directory() {
    let resumed = given::resumed(stopped(), Double::default());
    assert_eq!(resumed.count_of(ENVIRONMENT_OF_T1_BUILT), 1);
}

#[test]
fn a_resumed_directed_evolution_stopped_while_its_environments_are_built_again_stops_as_asked() {
    let docker = Double::default().answering(&["build"], &[Answer::Stopped]);
    assert!(matches!(
        given::resumed(stopped(), docker).result,
        Err(DirectedEvolutionError::Asked)
    ));
}
