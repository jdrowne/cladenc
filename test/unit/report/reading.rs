use std::fs;
use std::path::PathBuf;

use cladenc::genealogy::{EvaluationOutcome, Genealogy};
use cladenc::journal::{self, Journal, JournalError, Record, Settings};
use cladenc::report::{self, Report};
use tempfile::TempDir;

const CUT_SHORT: &str = "{\"record\":\"procreation_fin";

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/progenitor"),
        tasks: vec!["t1".to_owned()],
        docker_image: "image".to_owned(),
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
    }
}

fn run_directory(last_line: &str) -> TempDir {
    let run_directory = tempfile::tempdir().unwrap();
    let records = [
        Record::Run {
            format: journal::FORMAT,
            settings: settings(),
        },
        Record::EvaluationStarted {
            harness: 0,
            task: "t1".to_owned(),
        },
        Record::EvaluationFinished {
            harness: 0,
            task: "t1".to_owned(),
            outcome: EvaluationOutcome::Success,
        },
        Record::ProcreationStarted {
            child: 1,
            parent: 0,
        },
    ];
    let lines: String = records
        .iter()
        .map(|record| record.line().unwrap() + "\n")
        .collect();
    fs::write(journal::path(run_directory.path()), lines + last_line).unwrap();
    run_directory
}

fn of_the_records() -> Report {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "t1").unwrap();
    genealogy
        .end_evaluation(0, "t1", EvaluationOutcome::Success)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    Report::of(&settings(), &genealogy)
}

#[test]
fn a_report_is_read_from_the_journal_of_a_run_directory() {
    assert_eq!(
        report::read(run_directory("").path()).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_directory_that_holds_no_journal_cannot_be_reported_on() {
    let run_directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        report::read(run_directory.path()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_last_line_that_was_cut_short_is_ignored() {
    assert_eq!(
        report::read(run_directory(CUT_SHORT).path()).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_last_line_that_was_cut_short_is_left_where_it_is() {
    let run_directory = run_directory(CUT_SHORT);
    let _report = report::read(run_directory.path());
    assert!(
        fs::read_to_string(journal::path(run_directory.path()))
            .unwrap()
            .ends_with(CUT_SHORT)
    );
}

#[test]
fn a_report_is_read_while_the_journal_is_locked() {
    let run_directory = run_directory("");
    let _in_use = Journal::open_existing(run_directory.path()).unwrap();
    assert!(report::read(run_directory.path()).is_ok());
}
