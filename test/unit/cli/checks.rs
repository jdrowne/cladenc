use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
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
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: test_inputs::benchmark(directory, &["t2", "t1"]),
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

fn with_no_check_in_a_task() -> Refused {
    checked(|_, options| fs::remove_file(options.benchmark.join("t2/check")).unwrap()).unwrap_err()
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
            model: "a-model".to_owned(),
            model_url: "http://models.example/v1".to_owned(),
            model_key_file: None,
            benchmark: root.join("benchmark"),
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
fn a_benchmark_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| options.benchmark = directory.path().join("missing")),
        "--benchmark"
    );
}

#[test]
fn a_benchmark_that_does_not_exist_is_said_not_to_be_a_directory() {
    let refused = checked(|directory, options| {
        options.benchmark = directory.path().join("missing");
    })
    .unwrap_err();
    assert_eq!(
        refused.reason.as_deref(),
        Some("what was given is not a directory that can be read")
    );
}

#[test]
fn a_benchmark_that_is_one_task_and_not_a_directory_of_tasks_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.benchmark = options.benchmark.join("t1")),
        "--benchmark"
    );
}

#[test]
fn a_benchmark_that_is_one_task_is_said_to_be_one_task() {
    let refused = checked(|_, options| options.benchmark = options.benchmark.join("t1"));
    assert_eq!(
        refused.unwrap_err().reason.as_deref(),
        Some(
            "what was given is the directory of one task, and a benchmark is the directory that \
             holds the tasks"
        )
    );
}

#[test]
fn a_benchmark_with_a_task_that_cannot_be_used_is_refused() {
    assert_eq!(with_no_check_in_a_task().option, "--benchmark");
}

#[test]
fn a_benchmark_that_is_refused_for_a_task_names_the_task_and_what_it_lacks() {
    assert_eq!(
        with_no_check_in_a_task().reason.as_deref(),
        Some("task t2 holds no executable file named check")
    );
}

#[test]
fn a_benchmark_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|_, options| {
        options.benchmark = PathBuf::from("test/unit/samples/benchmark");
    })
    .unwrap();
    assert!(settings.benchmark.is_absolute());
}

#[test]
fn the_tasks_of_a_search_are_the_tasks_its_benchmark_holds() {
    let settings = checked(|_, options| {
        options.benchmark = PathBuf::from("test/unit/samples/benchmark");
    })
    .unwrap();
    assert_eq!(settings.tasks, ["add-a-flag", "fix-the-parser"]);
}

#[test]
fn a_benchmark_given_through_a_symbolic_link_is_the_directory_it_leads_to() {
    let settings = checked(|directory, options| {
        let link = directory.path().join("link");
        symlink(&options.benchmark, &link).unwrap();
        options.benchmark = link;
    })
    .unwrap();
    assert!(settings.benchmark.ends_with("benchmark"));
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
        reason: None,
    };
    assert_eq!(refused.to_string(), "--evaluator takes an executable file");
}

#[test]
fn a_value_refused_for_a_reason_is_said_with_the_reason_after_what_the_option_takes() {
    assert_eq!(
        with_no_check_in_a_task().to_string(),
        "--benchmark takes the directory of a benchmark, in which every directory is a task that \
         holds a file of text named statement, a directory named files and an executable file \
         named check: task t2 holds no executable file named check"
    );
}

#[test]
fn a_value_refused_for_what_its_option_takes_alone_has_no_reason() {
    let refused = checked(|directory, options| {
        options.progenitor = directory.path().join("missing");
    })
    .unwrap_err();
    assert_eq!(refused.reason, None);
}

#[test]
fn the_model_and_its_address_are_settings_as_they_were_given() {
    let settings = checked(|_, options| {
        options.model = "llama3.3:70b".to_owned();
        options.model_url = "http://host.docker.internal:11434".to_owned();
    })
    .unwrap();
    assert_eq!(
        (settings.model, settings.model_url),
        (
            "llama3.3:70b".to_owned(),
            "http://host.docker.internal:11434".to_owned()
        )
    );
}

#[test]
fn the_file_of_the_key_is_a_setting_and_the_key_is_not() {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    let file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    options.model_key_file = Some(file.clone());
    assert_eq!(options.settings().unwrap().model_key_file, Some(file));
}

#[test]
fn a_file_of_the_key_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|_, options| {
        options.model_key_file = Some(PathBuf::from("test/unit/samples/model-key"));
    })
    .unwrap();
    assert!(settings.model_key_file.unwrap().is_absolute());
}

#[test]
fn a_file_of_the_key_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            options.model_key_file = Some(directory.path().join("missing"));
        }),
        "--model-key-file"
    );
}

#[test]
fn a_file_of_the_key_that_holds_no_key_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            options.model_key_file = Some(test_inputs::model_key_file(directory.path(), "\n"));
        }),
        "--model-key-file"
    );
}

#[test]
fn a_file_of_the_key_with_no_path_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.model_key_file = Some(PathBuf::new())),
        "--model-key-file"
    );
}
