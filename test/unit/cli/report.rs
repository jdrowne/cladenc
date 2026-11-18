use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::cli::{self, Cli, Command, ReportOptions};
use cladenc::journal::{self, Record, Settings};
use cladenc::report;
use clap::Parser;
use clap::error::ErrorKind;

struct Ran {
    status: u8,
    output: String,
    errors: String,
}

fn parsed(options: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(["cladenc", "report"].iter().chain(options).copied())
}

fn ran(outputs_dir: PathBuf, json: bool) -> Ran {
    let cli = Cli {
        command: Command::Report(ReportOptions { outputs_dir, json }),
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

fn reported(json: bool) -> (Ran, String) {
    let outputs_directory = test_inputs::temporary_directory();
    let records = [
        Record::Run {
            format: journal::FORMAT,
            settings: Box::new(Settings {
                primogenitor: PathBuf::from("/primogenitor"),
                model: "a-model".to_owned(),
                model_url: "http://models.example/v1".to_owned(),
                model_key_file: None,
                benchmarks: vec![Benchmark {
                    directory: PathBuf::from("/abs/benchmark"),
                    tasks: vec!["t1".to_owned()],
                }],
                docker_image: "image".to_owned(),
                dockerfile: None,
                docker_build_timeout: None,
                harness_build_command: None,
                harness_run_command: "run a harness".to_owned(),
                max_procreations: 800,
                genealogy_growth: 0.6,
                max_concurrent_containers: 1,
                procreation_timeout: 60,
                attempt_timeout: 60,
                holdout_percentage: 50.0,
                holdout_tasks: Vec::new(),
                seed: 7,
            }),
        },
        Record::ProcreationStarted {
            child: 1,
            progenitor: 0,
        },
    ];
    let lines: String = records
        .iter()
        .map(|record| record.line().unwrap() + "\n")
        .collect();
    fs::write(journal::path(outputs_directory.path()), lines).unwrap();
    let object = report::read(outputs_directory.path()).unwrap().json();
    (ran(outputs_directory.path().to_owned(), json), object)
}

fn missing() -> PathBuf {
    test_inputs::temporary_directory().path().join("missing")
}

#[test]
fn the_outputs_directory_is_the_one_given() {
    assert!(matches!(
        parsed(&["--outputs-dir=/outputs"]).unwrap().command,
        Command::Report(ReportOptions { outputs_dir, .. }) if outputs_dir == Path::new("/outputs")
    ));
}

#[test]
fn leaving_out_the_outputs_directory_is_an_error() {
    assert_eq!(
        parsed(&[]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn json_takes_no_value() {
    assert!(parsed(&["--outputs-dir=/outputs", "--json=yes"]).is_err());
}

#[test]
fn writing_json_turns_it_on() {
    assert!(matches!(
        parsed(&["--outputs-dir=/outputs", "--json"])
            .unwrap()
            .command,
        Command::Report(ReportOptions { json: true, .. })
    ));
}

#[test]
fn leaving_json_out_leaves_it_off() {
    assert!(matches!(
        parsed(&["--outputs-dir=/outputs"]).unwrap().command,
        Command::Report(ReportOptions { json: false, .. })
    ));
}

#[test]
fn a_report_prints_the_table_on_standard_output() {
    let (ran, _object) = reported(false);
    assert_eq!(
        ran.output,
        "model a-model at http://models.example/v1\n\
         started 1 of 800 procreations, best harness 0\n\
         \n\
         id  progenitor  state          holdout completed  holdout not completed  other completed  other not completed  clade completed  clade not completed  clade score  attempt seconds  procreation seconds\n\
         0   -           created        0                  0                      0                0                    0                0                    -            0.0              -\n\
         1   0           being created  0                  0                      0                0                    0                0                    -            0.0              -\n"
    );
}

#[test]
fn with_json_a_report_prints_the_object_on_one_line() {
    let (ran, object) = reported(true);
    assert_eq!(ran.output, format!("{object}\n"));
}

#[test]
fn a_report_gives_exit_status_0() {
    let (ran, _object) = reported(false);
    assert_eq!(ran.status, 0);
}

#[test]
fn a_report_prints_nothing_on_standard_error() {
    let (ran, _object) = reported(false);
    assert_eq!(ran.errors, "");
}

#[test]
fn an_outputs_directory_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(missing(), false).status, 2);
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_on_standard_error() {
    assert!(
        ran(missing(), false)
            .errors
            .starts_with("cladenc: the outputs directory cannot be used: ")
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_with_the_system_s_reason() {
    assert_eq!(
        ran(missing(), false).errors,
        "cladenc: the outputs directory cannot be used: No such file or directory (os error 2)\n"
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_prints_nothing_on_standard_output() {
    assert_eq!(ran(missing(), false).output, "");
}

#[test]
fn an_outputs_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(PathBuf::new(), false).status, 2);
}
