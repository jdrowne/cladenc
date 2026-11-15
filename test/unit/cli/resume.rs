use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli, Command, ResumeOptions};
use cladenc::journal::{self, Record, Settings};
use clap::Parser;
use clap::error::ErrorKind;
use tempfile::TempDir;

struct Ran {
    status: u8,
    output: String,
    errors: String,
}

fn parsed(options: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(["cladenc", "resume"].iter().chain(options).copied())
}

fn ran(run_dir: PathBuf, quiet: bool) -> Ran {
    let cli = Cli {
        command: Command::Resume(ResumeOptions { run_dir, quiet }),
    };
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let status = cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut errors);
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}

fn missing() -> PathBuf {
    test_inputs::temporary_directory().path().join("missing")
}

fn started_with_a_key_whose_file_is_gone() -> (TempDir, PathBuf) {
    let run_directory = test_inputs::temporary_directory();
    let file = run_directory.path().join("missing");
    let run = Record::Run {
        format: journal::FORMAT,
        settings: Box::new(Settings {
            progenitor: PathBuf::from("/progenitor"),
            model: "a-model".to_owned(),
            model_url: "http://models.example/v1".to_owned(),
            model_key_file: Some(file.clone()),
            benchmark: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned()],
            docker_image: "image".to_owned(),
            dockerfile: None,
            docker_build_timeout: None,
            harness_build_command: None,
            harness_run_command: "run a harness".to_owned(),
            evaluator: PathBuf::from("/evaluator"),
            budget: 800,
            genealogy_growth: 0.6,
            confidence: 99.0,
            initial_procreations: 0,
            workers: 1,
            procreation_timeout: 60,
            evaluation_timeout: 60,
            seed: 7,
        }),
    };
    fs::write(
        journal::path(run_directory.path()),
        run.line().unwrap() + "\n",
    )
    .unwrap();
    (run_directory, file)
}

#[test]
fn a_run_directory_with_a_colon_in_its_path_is_refused_before_it_is_looked_at() {
    assert!(
        ran(PathBuf::from("/first:second"), false)
            .errors
            .starts_with("cladenc: --run-dir takes a path that holds no colon")
    );
}

#[test]
fn the_run_directory_is_the_one_given() {
    assert!(matches!(
        parsed(&["--run-dir=/run"]).unwrap().command,
        Command::Resume(ResumeOptions { run_dir, .. }) if run_dir == Path::new("/run")
    ));
}

#[test]
fn leaving_out_the_run_directory_is_an_error() {
    assert_eq!(
        parsed(&[]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn quiet_takes_no_value() {
    assert!(parsed(&["--run-dir=/run", "--quiet=yes"]).is_err());
}

#[test]
fn writing_quiet_turns_it_on() {
    assert!(matches!(
        parsed(&["--run-dir=/run", "--quiet"]).unwrap().command,
        Command::Resume(ResumeOptions { quiet: true, .. })
    ));
}

#[test]
fn leaving_quiet_out_leaves_it_off() {
    assert!(matches!(
        parsed(&["--run-dir=/run"]).unwrap().command,
        Command::Resume(ResumeOptions { quiet: false, .. })
    ));
}

#[test]
fn a_run_directory_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(missing(), false).status, 2);
}

#[test]
fn a_run_directory_that_does_not_exist_is_reported_on_standard_error() {
    assert!(
        ran(missing(), false)
            .errors
            .starts_with("cladenc: the run directory cannot be used: ")
    );
}

#[test]
fn a_run_directory_that_does_not_exist_is_reported_with_the_system_s_reason() {
    assert_eq!(
        ran(missing(), false).errors,
        "cladenc: the run directory cannot be used: No such file or directory (os error 2)\n"
    );
}

#[test]
fn a_run_directory_that_does_not_exist_prints_nothing_on_standard_output() {
    assert_eq!(ran(missing(), false).output, "");
}

#[test]
fn with_quiet_a_run_directory_that_does_not_exist_prints_nothing_on_standard_error() {
    assert_eq!(ran(missing(), true).errors, "");
}

#[test]
fn a_search_whose_file_of_the_key_is_gone_gives_exit_status_2() {
    let (run_directory, _file) = started_with_a_key_whose_file_is_gone();
    assert_eq!(ran(run_directory.path().to_owned(), false).status, 2);
}

#[test]
fn a_search_whose_file_of_the_key_is_gone_is_reported_with_the_file() {
    let (run_directory, file) = started_with_a_key_whose_file_is_gone();
    assert_eq!(
        ran(run_directory.path().to_owned(), false).errors,
        format!(
            "cladenc: the key of the model cannot be read: {} is not a file that holds a key on \
             one line and nothing else\n",
            file.display()
        )
    );
}

#[test]
fn a_run_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(PathBuf::new(), false).status, 2);
}
