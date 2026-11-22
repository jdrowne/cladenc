use std::fs;
use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::AttemptResult;
use crate::journal::{self, Journal, JournalError, Record, Seconds, Settings};
use crate::report::leader::Leader;
use crate::report::{self, Report};
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
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned(), "benchmark/t2".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["benchmark/t2".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
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
            task: "benchmark/t1".to_owned(),
        },
        Record::AttemptFinished {
            harness: 0,
            task: "benchmark/t1".to_owned(),
            result: AttemptResult::Completed,
            seconds: 1.5,
        },
        Record::AttemptStarted {
            harness: 0,
            task: "benchmark/t2".to_owned(),
        },
        Record::AttemptFinished {
            harness: 0,
            task: "benchmark/t2".to_owned(),
            result: AttemptResult::NotCompleted,
            seconds: 2.0,
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

fn finished_with(best_harness: u64) -> String {
    Record::DirectedEvolutionFinished { best_harness }
        .line()
        .unwrap()
        + "\n"
}

fn of_the_records() -> Report {
    let mut genealogy = settings().genealogy();
    genealogy.start_attempt(0, "benchmark/t1").unwrap();
    genealogy
        .end_attempt(0, "benchmark/t1", AttemptResult::Completed)
        .unwrap();
    genealogy.start_attempt(0, "benchmark/t2").unwrap();
    genealogy
        .end_attempt(0, "benchmark/t2", AttemptResult::NotCompleted)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    let seconds = [
        Seconds {
            attempts: 3.5,
            procreation: None,
        },
        Seconds::default(),
    ];
    Report::of(&settings(), &genealogy, &seconds, None)
}

#[test]
fn a_report_is_read_from_the_journal_of_an_outputs_directory() {
    assert_eq!(
        report::read(outputs_directory("").path()).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_report_read_from_a_journal_counts_the_selection_tasks_its_run_record_leaves() {
    let report = report::read(outputs_directory("").path()).unwrap();
    assert_eq!(report.harnesses[0].selection_completed, 1);
}

#[test]
fn a_report_read_from_a_journal_counts_the_corroboration_tasks_its_run_record_names() {
    let report = report::read(outputs_directory("").path()).unwrap();
    assert_eq!(report.harnesses[0].corroboration_not_completed, 1);
}

#[test]
fn a_report_read_from_a_journal_holds_the_seconds_its_records_hold() {
    let report = report::read(outputs_directory("").path()).unwrap();
    assert_eq!(report.harnesses[0].attempt_seconds, 3.5);
}

#[test]
fn a_journal_that_does_not_say_that_the_directed_evolution_finished_is_of_one_that_has_not_ended() {
    assert!(
        !report::read(outputs_directory("").path())
            .unwrap()
            .has_ended()
    );
}

#[test]
fn a_journal_that_says_that_the_directed_evolution_finished_is_of_one_that_has_ended() {
    assert!(
        report::read(outputs_directory(&finished_with(0)).path())
            .unwrap()
            .has_ended()
    );
}

#[test]
fn the_best_harness_of_a_report_read_from_a_journal_is_the_one_its_last_record_names() {
    let report = report::read(outputs_directory(&finished_with(1)).path()).unwrap();
    assert_eq!(report.leader, Leader::BestHarness(1));
}

#[test]
fn the_harness_ahead_so_far_of_a_journal_that_has_not_ended_is_counted_from_its_records() {
    let report = report::read(outputs_directory("").path()).unwrap();
    assert_eq!(report.leader, Leader::HarnessAheadSoFar(0));
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
