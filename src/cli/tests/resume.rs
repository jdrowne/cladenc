use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::benchmark::Benchmark;
use crate::cli::{self, Cli, Command, ResumeOptions};
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
    Cli::try_parse_from(["cladenc", "resume"].iter().chain(options).copied())
}

fn ran(files: &Memory, outputs_dir: PathBuf) -> Ran {
    let cli = Cli {
        command: Command::Resume(ResumeOptions { outputs_dir }),
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

fn missing() -> PathBuf {
    Memory::default().path().join("missing")
}

#[test]
fn an_outputs_directory_with_a_colon_in_its_path_is_refused_before_it_is_looked_at() {
    assert!(
        ran(&Memory::default(), PathBuf::from("/first:second"))
            .errors
            .starts_with("cladenc: --outputs-dir takes a path that holds no colon")
    );
}

#[test]
fn the_outputs_directory_is_the_one_given() {
    assert!(matches!(
        parsed(&["--outputs-dir=/outputs"]).unwrap().command,
        Command::Resume(ResumeOptions { outputs_dir }) if outputs_dir == Path::new("/outputs")
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
fn quiet_is_no_flag_of_resume() {
    assert_eq!(
        parsed(&["--outputs-dir=/outputs", "--quiet"])
            .unwrap_err()
            .kind(),
        ErrorKind::UnknownArgument
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(&Memory::default(), missing()).status, 2);
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_on_standard_error() {
    assert!(
        ran(&Memory::default(), missing())
            .errors
            .starts_with("cladenc: the outputs directory cannot be used: ")
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_is_reported_with_the_system_s_reason() {
    assert_eq!(
        ran(&Memory::default(), missing()).errors,
        "cladenc: the outputs directory cannot be used: No such file or directory (os error 2)\n"
    );
}

#[test]
fn an_outputs_directory_that_does_not_exist_prints_nothing_on_standard_output() {
    assert_eq!(ran(&Memory::default(), missing()).output, "");
}

#[test]
fn an_outputs_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(ran(&Memory::default(), PathBuf::new()).status, 2);
}

fn ended() -> Memory {
    let outputs_directory = Memory::default();
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
                max_procreations: 1,
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
        Record::ProcreationFinished {
            child: 1,
            outcome: CreationOutcome::Failed,
            seconds: 30.26,
        },
        Record::DirectedEvolutionFinished { best_harness: 0 },
    ];
    let lines: String = records
        .iter()
        .map(|record| record.line().unwrap() + "\n")
        .collect();
    outputs_directory
        .write(journal::path(outputs_directory.path()), lines)
        .unwrap();
    outputs_directory
}

fn best_harness_of(outputs_directory: &Memory) -> String {
    outputs_directory
        .path()
        .join("harnesses/0")
        .display()
        .to_string()
}

#[test]
fn resuming_a_directed_evolution_that_had_ended_gives_exit_status_0() {
    let outputs_directory = ended();
    assert_eq!(
        ran(&outputs_directory, outputs_directory.path().to_owned()).status,
        0
    );
}

#[test]
fn resuming_a_directed_evolution_that_had_ended_prints_only_the_directory_of_its_best_harness() {
    let outputs_directory = ended();
    assert_eq!(
        ran(&outputs_directory, outputs_directory.path().to_owned()).output,
        format!("{}\n", best_harness_of(&outputs_directory))
    );
}

#[test]
fn resuming_a_directed_evolution_that_had_ended_says_first_on_standard_error_that_it_had_finished()
{
    let outputs_directory = ended();
    assert_eq!(
        ran(&outputs_directory, outputs_directory.path().to_owned())
            .errors
            .lines()
            .next(),
        Some(
            format!(
                "cladenc: the directed evolution had finished: its best harness is harness 0, \
                 kept in {}",
                best_harness_of(&outputs_directory)
            )
            .as_str()
        )
    );
}

#[test]
fn resuming_a_directed_evolution_that_had_ended_prints_the_report_after_that_line() {
    let outputs_directory = ended();
    let errors = ran(&outputs_directory, outputs_directory.path().to_owned()).errors;
    assert_eq!(
        errors.split_once('\n').map(|(_, report)| report.to_owned()),
        Some(
            report::read(&outputs_directory, outputs_directory.path())
                .unwrap()
                .table()
        )
    );
}

#[test]
fn the_report_that_resume_prints_names_the_best_harness() {
    let outputs_directory = ended();
    assert_eq!(
        ran(&outputs_directory, outputs_directory.path().to_owned())
            .errors
            .lines()
            .nth(2),
        Some("started 1 of 1 procreation, best harness 0")
    );
}

#[test]
fn a_directed_evolution_that_cannot_be_resumed_prints_no_report() {
    assert_eq!(ran(&Memory::default(), missing()).errors.lines().count(), 1);
}
