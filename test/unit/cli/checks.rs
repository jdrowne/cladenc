use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use cladenc::cli::{Refused, StartOptions};
use cladenc::journal::Settings;
use tempfile::TempDir;

fn program(directory: &Path, name: &str, mode: u32) -> PathBuf {
    let program = directory.join(name);
    fs::write(&program, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(mode)).unwrap();
    program
}

fn options(directory: &Path) -> StartOptions {
    StartOptions {
        progenitor: test_inputs::progenitor(directory),
        tasks: test_inputs::task_file(directory, &["t1", "t2"]),
        docker_image: "image".to_owned(),
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        evaluator: program(directory, "evaluate", 0o755),
        run_dir: directory.join("run"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 5,
        workers: 8,
        procreation_timeout: 3600,
        evaluation_timeout: 1800,
        seed: 7,
        quiet: false,
    }
}

fn checked(change: impl FnOnce(&TempDir, &mut StartOptions)) -> Result<Settings, Refused> {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    change(&directory, &mut options);
    options.settings()
}

fn option_refused(change: impl FnOnce(&TempDir, &mut StartOptions)) -> &'static str {
    checked(change).unwrap_err().option
}

fn with_tasks(lines: &'static [&'static str]) -> Result<Settings, Refused> {
    checked(|directory, options| {
        options.tasks = test_inputs::task_file(directory.path(), lines);
    })
}

#[test]
fn values_that_fit_their_rows_give_the_settings_of_the_run() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let root = fs::canonicalize(directory.path()).unwrap();
    assert_eq!(
        options.settings(),
        Ok(Settings {
            progenitor: root.join("progenitor"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
            docker_image: "image".to_owned(),
            harness_build_command: None,
            harness_run_command: "run a harness".to_owned(),
            evaluator: root.join("evaluate"),
            budget: 800,
            genealogy_growth: 0.6,
            confidence: 99.0,
            initial_procreations: 5,
            workers: 8,
            procreation_timeout: 3600,
            evaluation_timeout: 1800,
            seed: 7,
        })
    );
}

#[test]
fn a_path_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|_, options| options.progenitor = PathBuf::from("src")).unwrap();
    assert!(settings.progenitor.is_absolute());
}

#[test]
fn a_progenitor_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| options.progenitor = directory.path().join("missing")),
        "--progenitor"
    );
}

#[test]
fn a_progenitor_that_is_a_file_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.progenitor = options.evaluator.clone()),
        "--progenitor"
    );
}

#[test]
fn a_progenitor_that_holds_a_socket_is_refused() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let _socket = UnixListener::bind(options.progenitor.join("socket")).unwrap();
    assert_eq!(options.settings().unwrap_err().option, "--progenitor");
}

#[test]
fn a_task_file_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| options.tasks = directory.path().join("missing")),
        "--tasks"
    );
}

#[test]
fn a_task_file_with_no_line_is_refused() {
    assert_eq!(with_tasks(&[]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_file_with_a_blank_line_is_refused() {
    assert_eq!(with_tasks(&["t1", "", "t2"]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_id_with_a_space_is_refused() {
    assert_eq!(with_tasks(&["task one"]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_id_with_a_slash_is_refused() {
    assert_eq!(with_tasks(&["tasks/one"]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_id_that_is_a_dot_is_refused() {
    assert_eq!(with_tasks(&["."]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_id_that_is_two_dots_is_refused() {
    assert_eq!(with_tasks(&[".."]).unwrap_err().option, "--tasks");
}

#[test]
fn a_task_id_that_is_repeated_is_refused() {
    assert_eq!(
        with_tasks(&["t1", "t2", "t1"]).unwrap_err().option,
        "--tasks"
    );
}

#[test]
fn task_ids_of_letters_digits_dots_underscores_and_hyphens_are_accepted() {
    assert_eq!(
        with_tasks(&["Task-1.a_b", "...", "2"]).unwrap().tasks,
        ["Task-1.a_b", "...", "2"]
    );
}

#[test]
fn the_image_and_the_run_command_are_settings_as_they_were_given() {
    let settings = checked(|_, options| {
        options.docker_image = "registry.example/harness:1".to_owned();
        options.harness_run_command = "sh \"$CLADENC_HARNESS/harness.sh\"".to_owned();
    })
    .unwrap();
    assert_eq!(
        (settings.docker_image, settings.harness_run_command),
        (
            "registry.example/harness:1".to_owned(),
            "sh \"$CLADENC_HARNESS/harness.sh\"".to_owned()
        )
    );
}

#[test]
fn the_build_command_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| {
        options.harness_build_command = Some("make harness".to_owned());
    })
    .unwrap();
    assert_eq!(
        settings.harness_build_command,
        Some("make harness".to_owned())
    );
}

#[test]
fn an_evaluator_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| options.evaluator = directory.path().join("missing")),
        "--evaluator"
    );
}

#[test]
fn an_evaluator_that_is_not_executable_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            options.evaluator = program(directory.path(), "plain", 0o644);
        }),
        "--evaluator"
    );
}

#[test]
fn an_evaluator_that_is_a_directory_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.evaluator = options.progenitor.clone()),
        "--evaluator"
    );
}

#[test]
fn a_refused_value_is_said_with_its_option_and_what_the_option_takes() {
    let refused = Refused {
        option: "--evaluator",
        accepted: "an executable file",
    };
    assert_eq!(refused.to_string(), "--evaluator takes an executable file");
}
