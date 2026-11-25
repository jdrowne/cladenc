use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::benchmark::Benchmark;
use crate::cli::{self, Cli, Command, ReportOptions};
use crate::containers::tests::double::Double;
use crate::file_system::tests::double::Memory;
use crate::genealogy::CreationOutcome;
use crate::journal::{self, Record, Settings};
use crate::report;
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

fn ran(files: &Memory, outputs_dir: PathBuf, json: bool) -> Ran {
    let cli = Cli {
        command: Command::Report(ReportOptions { outputs_dir, json }),
    };
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let status = cli::execute(
        Double::default().on(files).outside(),
        &cli,
        &AtomicBool::new(false),
        &mut output,
        &mut errors,
    );
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}

fn started() -> Vec<Record> {
    vec![
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
                max_concurrent_containers: 1,
                procreation_timeout: 60,
                max_verifier_health_check_retries: 0,
                max_task_health_check_retries: 0,
                selection_tasks_percentage: 25.0,
                holdout_tasks: Vec::new(),
                corroboration_tasks_percentage: 25.0,
                corroboration_tasks: Vec::new(),
                environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
                seed: 7,
            }),
        },
        Record::ProcreationStarted {
            child: 1,
            progenitor: 0,
        },
    ]
}

fn ended() -> Vec<Record> {
    let mut records = started();
    records.push(Record::ProcreationFinished {
        child: 1,
        outcome: CreationOutcome::Failed,
        seconds: 30.26,
    });
    records.push(Record::DirectedEvolutionFinished { best_harness: 0 });
    records
}

fn outputs_directory_of(records: &[Record]) -> Memory {
    let outputs_directory = Memory::default();
    let lines: String = records
        .iter()
        .map(|record| record.line().unwrap() + "\n")
        .collect();
    outputs_directory
        .write(journal::path(outputs_directory.path()), lines)
        .unwrap();
    outputs_directory
}

fn reported(json: bool) -> (Ran, String) {
    let outputs_directory = outputs_directory_of(&ended());
    let object = report::read(&outputs_directory, outputs_directory.path())
        .unwrap()
        .json();
    (
        ran(
            &outputs_directory,
            outputs_directory.path().to_owned(),
            json,
        ),
        object,
    )
}

fn reported_before_the_end() -> Ran {
    let outputs_directory = outputs_directory_of(&started());
    ran(
        &outputs_directory,
        outputs_directory.path().to_owned(),
        false,
    )
}

fn missing() -> PathBuf {
    Memory::default().path().join("missing")
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
         completed by at least one harness: selection tasks 0 of 0, corroboration tasks 0 of 0, non-holdout tasks 0 of 1\n\
         on the corroboration tasks: the best harness is the primogenitor, so there are no two harnesses to compare\n\
         experiments: conducted 0, the progenitor's child created in 0, the singleton control group harness created in 0, both created in 0\n\
         on the corroboration tasks of the experiments: no experiment created both the progenitor's child and the singleton control group harness, so there are no two harnesses to compare\n\
         \n\
         id  progenitor  experiment  state    selection completed  selection not completed  corroboration completed  corroboration not completed  non-holdout completed  non-holdout not completed  clade completed  clade not completed  clade score  trial seconds  procreation seconds\n\
         0   -           -           created  0                    0                        0                        0                            0                      0                          0                0                    -            0.0            -\n\
         1   0           -           failed   0                    0                        0                        0                            0                      0                          0                0                    -            0.0            30.3\n"
    );
}

#[test]
fn with_json_a_report_prints_the_object_on_one_line() {
    let (ran, object) = reported(true);
    assert_eq!(ran.output, format!("{object}\n"));
}

#[test]
fn with_json_a_report_says_first_that_the_directed_evolution_has_ended() {
    let (ran, _object) = reported(true);
    assert!(ran.output.starts_with(r#"{"ended":true,"#));
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
fn a_directed_evolution_that_has_not_ended_gives_exit_status_2() {
    assert_eq!(reported_before_the_end().status, 2);
}

#[test]
fn a_directed_evolution_that_has_not_ended_is_refused_with_the_sub_command_to_call() {
    assert_eq!(
        reported_before_the_end().errors,
        "cladenc: the directed evolution has not ended: call `cladenc status` to read its state \
         so far\n"
    );
}

#[test]
fn a_directed_evolution_that_has_not_ended_prints_nothing_on_standard_output() {
    assert_eq!(reported_before_the_end().output, "");
}

#[test]
fn an_outputs_directory_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(&Memory::default(), missing(), false).status, 2);
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_on_standard_error() {
    assert!(
        ran(&Memory::default(), missing(), false)
            .errors
            .starts_with("cladenc: the outputs directory cannot be used: ")
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_with_the_system_s_reason() {
    assert_eq!(
        ran(&Memory::default(), missing(), false).errors,
        "cladenc: the outputs directory cannot be used: No such file or directory (os error 2)\n"
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_prints_nothing_on_standard_output() {
    assert_eq!(ran(&Memory::default(), missing(), false).output, "");
}

#[test]
fn an_outputs_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(&Memory::default(), PathBuf::new(), false).status, 2);
}

#[test]
fn a_report_leaves_the_journal_unchanged() {
    let outputs_directory = outputs_directory_of(&ended());
    let journal = journal::path(outputs_directory.path());
    let before = outputs_directory.read_to_string(&journal).unwrap();
    let _shown = ran(
        &outputs_directory,
        outputs_directory.path().to_owned(),
        false,
    );
    assert_eq!(outputs_directory.read_to_string(&journal).unwrap(), before);
}
