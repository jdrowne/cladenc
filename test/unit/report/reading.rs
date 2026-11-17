use std::fs;
use std::path::PathBuf;

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::{AttemptResult, Genealogy};
use cladenc::journal::{self, Journal, JournalError, Record, Settings};
use cladenc::report::{self, Report};
use tempfile::TempDir;

const CUT_SHORT: &str = "{\"record\":\"procreation_fin";

fn settings() -> Settings {
    Settings {
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
        seed: 7,
    }
}

fn outputs_directory(last_line: &str) -> TempDir {
    let outputs_directory = tempfile::tempdir().unwrap();
    let records = [
        Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings()),
        },
        Record::AttemptStarted {
            harness: 0,
            task: "t1".to_owned(),
        },
        Record::AttemptFinished {
            harness: 0,
            task: "t1".to_owned(),
            result: AttemptResult::Completed,
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
    fs::write(journal::path(outputs_directory.path()), lines + last_line).unwrap();
    outputs_directory
}

fn of_the_records() -> Report {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "t1").unwrap();
    genealogy
        .end_attempt(0, "t1", AttemptResult::Completed)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    Report::of(&settings(), &genealogy)
}

#[test]
fn a_report_is_read_from_the_journal_of_an_outputs_directory() {
    assert_eq!(
        report::read(outputs_directory("").path()).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_directory_that_holds_no_journal_cannot_be_reported_on() {
    let outputs_directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        report::read(outputs_directory.path()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_last_line_that_was_cut_short_is_ignored() {
    assert_eq!(
        report::read(outputs_directory(CUT_SHORT).path()).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_last_line_that_was_cut_short_is_left_where_it_is() {
    let outputs_directory = outputs_directory(CUT_SHORT);
    let _report = report::read(outputs_directory.path());
    assert!(
        fs::read_to_string(journal::path(outputs_directory.path()))
            .unwrap()
            .ends_with(CUT_SHORT)
    );
}

#[test]
fn a_report_is_read_while_the_journal_is_locked() {
    let outputs_directory = outputs_directory("");
    let _in_use = Journal::open_existing(outputs_directory.path()).unwrap();
    assert!(report::read(outputs_directory.path()).is_ok());
}
