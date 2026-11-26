use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::file_system::tests::double::Memory;
use crate::genealogy::TrialResult;
use crate::journal::{self, Journal, JournalError, Record, Seconds, Settings};
use crate::report::leader::Leader;
use crate::report::{self, Report};

const CUT_SHORT: &str = "{\"record\":\"diversification_fin";

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
        max_diversifications: 800,
        max_concurrent_containers: 1,
        diversification_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned(), "benchmark/t2".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["benchmark/t2".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn outputs_directory(last_line: &str) -> Memory {
    let directory = Memory::default();
    let records = [
        Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings()),
        },
        Record::TrialStarted {
            harness: 0,
            task: "benchmark/t1".to_owned(),
        },
        Record::TrialFinished {
            harness: 0,
            task: "benchmark/t1".to_owned(),
            result: TrialResult::Completed,
            seconds: 1.5,
        },
        Record::TrialStarted {
            harness: 0,
            task: "benchmark/t2".to_owned(),
        },
        Record::TrialFinished {
            harness: 0,
            task: "benchmark/t2".to_owned(),
            result: TrialResult::NotCompleted,
            seconds: 2.0,
        },
        Record::DiversificationStarted {
            child: 1,
            progenitor: 0,
        },
    ];
    let lines: String = records
        .iter()
        .map(|record| record.line().unwrap() + "\n")
        .collect();
    directory
        .write(journal::path(directory.path()), lines + last_line)
        .unwrap();
    directory
}

fn read(directory: &Memory) -> Result<Report, JournalError> {
    report::read(directory, directory.path())
}

fn finished_with(best_harness: u64) -> String {
    Record::DirectedEvolutionFinished { best_harness }
        .line()
        .unwrap()
        + "\n"
}

fn of_the_records() -> Report {
    let mut genealogy = settings().genealogy();
    genealogy.start_trial(0, "benchmark/t1").unwrap();
    genealogy
        .end_trial(0, "benchmark/t1", TrialResult::Completed)
        .unwrap();
    genealogy.start_trial(0, "benchmark/t2").unwrap();
    genealogy
        .end_trial(0, "benchmark/t2", TrialResult::NotCompleted)
        .unwrap();
    genealogy.start_diversification(0).unwrap();
    let seconds = [
        Seconds {
            trials: 3.5,
            diversification: None,
        },
        Seconds::default(),
    ];
    Report::of(&settings(), &genealogy, &seconds, None)
}

#[test]
fn a_report_is_read_from_the_journal_of_an_outputs_directory() {
    assert_eq!(read(&outputs_directory("")).unwrap(), of_the_records());
}

#[test]
fn a_report_read_from_a_journal_counts_the_selection_tasks_its_run_record_leaves() {
    let report = read(&outputs_directory("")).unwrap();
    assert_eq!(report.harnesses[0].selection_completed, 1);
}

#[test]
fn a_report_read_from_a_journal_counts_the_corroboration_tasks_its_run_record_names() {
    let report = read(&outputs_directory("")).unwrap();
    assert_eq!(report.harnesses[0].corroboration_not_completed, 1);
}

#[test]
fn a_report_read_from_a_journal_holds_the_seconds_its_records_hold() {
    let report = read(&outputs_directory("")).unwrap();
    assert_eq!(report.harnesses[0].trial_seconds, 3.5);
}

#[test]
fn a_journal_that_does_not_say_that_the_directed_evolution_finished_is_of_one_that_has_not_ended() {
    assert!(!read(&outputs_directory("")).unwrap().has_ended());
}

#[test]
fn a_journal_that_says_that_the_directed_evolution_finished_is_of_one_that_has_ended() {
    assert!(
        read(&outputs_directory(&finished_with(0)))
            .unwrap()
            .has_ended()
    );
}

#[test]
fn the_best_harness_of_a_report_read_from_a_journal_is_the_one_its_last_record_names() {
    let report = read(&outputs_directory(&finished_with(1))).unwrap();
    assert_eq!(report.leader, Leader::BestHarness(1));
}

#[test]
fn the_harness_ahead_so_far_of_a_journal_that_has_not_ended_is_counted_from_its_records() {
    let report = read(&outputs_directory("")).unwrap();
    assert_eq!(report.leader, Leader::HarnessAheadSoFar(0));
}

#[test]
fn a_directory_that_holds_no_journal_cannot_be_reported_on() {
    assert!(matches!(read(&Memory::default()), Err(JournalError::Io(_))));
}

#[test]
fn a_last_line_that_was_cut_short_is_ignored() {
    assert_eq!(
        read(&outputs_directory(CUT_SHORT)).unwrap(),
        of_the_records()
    );
}

#[test]
fn a_last_line_that_was_cut_short_is_left_where_it_is() {
    let directory = outputs_directory(CUT_SHORT);
    let _report = read(&directory);
    assert!(
        directory
            .read_to_string(journal::path(directory.path()))
            .unwrap()
            .ends_with(CUT_SHORT)
    );
}

#[test]
fn a_report_is_read_while_the_journal_is_locked() {
    let directory = outputs_directory("");
    let _in_use = Journal::open_existing(&directory, directory.path()).unwrap();
    assert!(read(&directory).is_ok());
}
