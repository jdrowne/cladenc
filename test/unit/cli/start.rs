use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli, Command, StartOptions};

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
        benchmark: vec![test_inputs::benchmark(directory.path(), &["t1", "t2"])],
        docker_image: Some("image".to_owned()),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.path().join("outputs"),
        max_procreations: 1,
        genealogy_growth: 0.0,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        attempt_timeout: 30,
        holdout_percentage: 50.0,
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
    options.docker_image = None;
    options.dockerfile = Some(directory.join("missing"));
    options.docker_build_timeout = Some(600);
}

fn with_a_holdout_percentage_that_holds_out_no_task(_: &Path, options: &mut StartOptions) {
    options.holdout_percentage = 10.0;
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
        "cladenc: --dockerfile takes an existing file\n"
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
fn a_holdout_percentage_that_holds_out_no_task_of_a_benchmark_gives_exit_status_2() {
    assert_eq!(
        ran(with_a_holdout_percentage_that_holds_out_no_task).status,
        2
    );
}

#[test]
fn a_holdout_percentage_that_holds_out_no_task_of_a_benchmark_is_refused_with_the_benchmark() {
    assert!(
        ran(with_a_holdout_percentage_that_holds_out_no_task)
            .errors
            .ends_with(
                ": the benchmark `benchmark` holds 2 tasks, and none of them would be held out\n"
            )
    );
}

#[test]
fn a_holdout_percentage_that_is_refused_is_refused_before_the_outputs_directory_is_made() {
    assert!(
        ran(with_a_holdout_percentage_that_holds_out_no_task)
            .errors
            .starts_with("cladenc: --holdout-percentage takes ")
    );
}
