use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::cli::{self, Cli, Command, StartOptions};

struct Ran {
    status: u8,
    output: String,
    errors: String,
}

fn ran(change: impl FnOnce(&Path, &mut StartOptions)) -> Ran {
    let directory = test_inputs::temporary_directory();
    let mut options = StartOptions {
        primogenitor: test_inputs::primogenitor(directory.path()),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark(
            directory.path(),
            &["t1", "t2", "t3"],
        )],
        procreation_docker_image: Some("image".to_owned()),
        procreation_dockerfile: None,
        procreation_docker_image_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.path().join("outputs"),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 30.0,
        corroboration_tasks_percentage: 30.0,
    };
    change(directory.path(), &mut options);
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let cli = Cli {
        command: Command::Start(Box::new(options)),
    };
    let status = cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut errors);
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}

fn with_a_missing_benchmark(directory: &Path, options: &mut StartOptions) {
    options.benchmark = vec![directory.join("missing")];
}

fn with_a_missing_file_of_the_key(directory: &Path, options: &mut StartOptions) {
    options.model_key_file = Some(directory.join("missing"));
}

fn with_a_missing_dockerfile(directory: &Path, options: &mut StartOptions) {
    options.procreation_docker_image = None;
    options.procreation_dockerfile = Some(directory.join("missing"));
    options.procreation_docker_image_build_timeout = Some(600);
}

fn with_a_percentage_that_leaves_no_selection_task(_: &Path, options: &mut StartOptions) {
    options.selection_tasks_percentage = 10.0;
}

fn with_a_percentage_that_leaves_no_corroboration_task(_: &Path, options: &mut StartOptions) {
    options.corroboration_tasks_percentage = 10.0;
}

fn in_an_outputs_directory_that_exists(directory: &Path, options: &mut StartOptions) {
    options.outputs_dir = directory.to_owned();
}

fn in_an_outputs_directory_with_a_colon_in_its_path(directory: &Path, options: &mut StartOptions) {
    options.outputs_dir = directory.join("first:second");
}

#[test]
fn a_refused_value_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_benchmark).status, 2);
}

#[test]
fn a_refused_value_is_reported_on_standard_error() {
    assert!(
        ran(with_a_missing_benchmark)
            .errors
            .starts_with("cladenc: --benchmark takes ")
    );
}

#[test]
fn a_refused_benchmark_is_reported_with_what_is_wrong_with_it() {
    assert!(
        ran(with_a_missing_benchmark)
            .errors
            .ends_with(": what was given is not a directory that can be read\n")
    );
}

#[test]
fn a_refused_value_prints_nothing_on_standard_output() {
    assert_eq!(ran(with_a_missing_benchmark).output, "");
}

#[test]
fn a_file_of_the_key_that_holds_no_key_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_file_of_the_key).status, 2);
}

#[test]
fn a_file_of_the_key_that_holds_no_key_is_refused_for_what_the_flag_takes() {
    assert_eq!(
        ran(with_a_missing_file_of_the_key).errors,
        "cladenc: --model-key-file takes a file that holds a key on one line and nothing else\n"
    );
}

#[test]
fn a_dockerfile_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_dockerfile).status, 2);
}

#[test]
fn a_dockerfile_that_does_not_exist_is_refused_for_what_the_flag_takes() {
    assert_eq!(
        ran(with_a_missing_dockerfile).errors,
        "cladenc: --procreation-dockerfile takes an existing file\n"
    );
}

#[test]
fn an_outputs_directory_at_which_something_exists_gives_exit_status_2() {
    assert_eq!(ran(in_an_outputs_directory_that_exists).status, 2);
}

#[test]
fn an_outputs_directory_at_which_something_exists_is_reported_last_on_standard_error() {
    assert!(
        ran(in_an_outputs_directory_that_exists)
            .errors
            .lines()
            .last()
            .is_some_and(|line| line.starts_with("cladenc: the outputs directory cannot be made: "))
    );
}

#[test]
fn an_outputs_directory_that_is_being_made_is_said_first_on_standard_error() {
    assert!(
        ran(in_an_outputs_directory_that_exists)
            .errors
            .starts_with("cladenc: making the outputs directory ")
    );
}

#[test]
fn an_outputs_directory_with_a_colon_in_its_path_gives_exit_status_2() {
    assert_eq!(
        ran(in_an_outputs_directory_with_a_colon_in_its_path).status,
        2
    );
}

#[test]
fn an_outputs_directory_with_a_colon_in_its_path_is_refused_for_what_docker_cannot_mount() {
    assert_eq!(
        ran(in_an_outputs_directory_with_a_colon_in_its_path).errors,
        "cladenc: --outputs-dir takes a path that holds no colon, because Docker cannot mount a \
         directory whose path holds one\n"
    );
}

#[test]
fn an_outputs_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(
        ran(|_, options| options.outputs_dir = PathBuf::new()).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_selection_task_gives_exit_status_2() {
    assert_eq!(
        ran(with_a_percentage_that_leaves_no_selection_task).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_selection_task_is_refused_with_the_benchmark() {
    assert!(
        ran(with_a_percentage_that_leaves_no_selection_task)
            .errors
            .ends_with(
                ": the benchmark `benchmark` holds 3 tasks, and none of them would be a selection \
                 task\n"
            )
    );
}

#[test]
fn a_percentage_that_leaves_no_selection_task_is_refused_before_the_outputs_directory_is_made() {
    assert!(
        ran(with_a_percentage_that_leaves_no_selection_task)
            .errors
            .starts_with("cladenc: --selection-tasks-percentage takes ")
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_corroboration_task_gives_exit_status_2() {
    assert_eq!(
        ran(with_a_percentage_that_leaves_no_corroboration_task).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_corroboration_task_is_refused_with_the_benchmark() {
    assert!(
        ran(with_a_percentage_that_leaves_no_corroboration_task)
            .errors
            .ends_with(
                ": the benchmark `benchmark` holds 3 tasks, and none of them would be a \
                 corroboration task\n"
            )
    );
}

#[test]
fn a_percentage_that_leaves_no_corroboration_task_is_refused_before_the_outputs_directory_is_made()
{
    assert!(
        ran(with_a_percentage_that_leaves_no_corroboration_task)
            .errors
            .starts_with("cladenc: --corroboration-tasks-percentage takes ")
    );
}
