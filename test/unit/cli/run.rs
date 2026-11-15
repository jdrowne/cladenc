use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli, Command, RunOptions};

struct Ran {
    status: u8,
    output: String,
    errors: String,
}

fn program(directory: &Path, name: &str) -> PathBuf {
    let program = directory.join(name);
    fs::write(&program, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    program
}

fn ran(change: impl FnOnce(&Path, &mut RunOptions)) -> Ran {
    let directory = test_inputs::temporary_directory();
    let mut options = RunOptions {
        progenitor: test_inputs::progenitor(directory.path()),
        tasks: test_inputs::task_file(directory.path(), &["t1"]),
        docker_image: "image".to_owned(),
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        evaluator: program(directory.path(), "evaluate"),
        run_dir: directory.path().join("run"),
        budget: 2,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_procreations: 0,
        workers: 1,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
        quiet: false,
    };
    change(directory.path(), &mut options);
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let cli = Cli {
        command: Command::Run(Box::new(options)),
    };
    let status = cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut errors);
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}

fn with_a_missing_task_file(directory: &Path, options: &mut RunOptions) {
    options.tasks = directory.join("missing");
}

fn in_a_run_directory_that_exists(directory: &Path, options: &mut RunOptions) {
    options.run_dir = directory.to_owned();
}

fn in_a_run_directory_with_a_colon_in_its_path(directory: &Path, options: &mut RunOptions) {
    options.run_dir = directory.join("first:second");
}

#[test]
fn a_refused_value_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_task_file).status, 2);
}

#[test]
fn a_refused_value_is_reported_on_standard_error() {
    assert!(
        ran(with_a_missing_task_file)
            .errors
            .starts_with("cladenc: --tasks takes ")
    );
}

#[test]
fn a_refused_value_prints_nothing_on_standard_output() {
    assert_eq!(ran(with_a_missing_task_file).output, "");
}

#[test]
fn with_quiet_a_refused_value_prints_nothing_on_standard_error() {
    let quiet = ran(|directory, options| {
        with_a_missing_task_file(directory, options);
        options.quiet = true;
    });
    assert_eq!(quiet.errors, "");
}

#[test]
fn a_run_directory_at_which_something_exists_gives_exit_status_2() {
    assert_eq!(ran(in_a_run_directory_that_exists).status, 2);
}

#[test]
fn a_run_directory_at_which_something_exists_is_reported_on_standard_error() {
    assert!(
        ran(in_a_run_directory_that_exists)
            .errors
            .starts_with("cladenc: the run directory cannot be made: ")
    );
}

#[test]
fn a_run_directory_with_a_colon_in_its_path_gives_exit_status_2() {
    assert_eq!(ran(in_a_run_directory_with_a_colon_in_its_path).status, 2);
}

#[test]
fn a_run_directory_with_a_colon_in_its_path_is_refused_for_what_docker_cannot_mount() {
    assert_eq!(
        ran(in_a_run_directory_with_a_colon_in_its_path).errors,
        "cladenc: --run-dir takes a path that holds no colon, because Docker cannot mount a \
         directory whose path holds one\n"
    );
}

#[test]
fn a_run_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(|_, options| options.run_dir = PathBuf::new()).status, 2);
}
