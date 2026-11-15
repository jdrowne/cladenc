use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use cladenc::genealogy::{EvaluationOutcome, Genealogy, ProcreationOutcome};
use cladenc::journal::{self, Journal, Record, Settings, Started, State};
use tempfile::TempDir;

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: PathBuf::from("/abs/benchmark"),
        tasks: vec!["t1".to_owned(), "t2".to_owned()],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        evaluator: PathBuf::from("/abs/evaluate"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 5,
        workers: 8,
        procreation_timeout: 3600,
        evaluation_timeout: 3600,
        seed: 7,
    }
}

fn procreation_started(child: u64, parent: u64) -> Record {
    Record::ProcreationStarted { child, parent }
}

fn procreation_finished(child: u64, outcome: ProcreationOutcome) -> Record {
    Record::ProcreationFinished { child, outcome }
}

fn evaluation_started(harness: u64, task: &str) -> Record {
    Record::EvaluationStarted {
        harness,
        task: task.to_owned(),
    }
}

fn evaluation_finished(harness: u64, task: &str, outcome: EvaluationOutcome) -> Record {
    Record::EvaluationFinished {
        harness,
        task: task.to_owned(),
        outcome,
    }
}

fn journal_of(records: &[Record]) -> TempDir {
    journal_with(settings(), records)
}

fn journal_with(settings: Settings, records: &[Record]) -> TempDir {
    let run_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(run_directory.path()).unwrap();
    journal
        .append(&Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        })
        .unwrap();
    for record in records {
        journal.append(record).unwrap();
    }
    run_directory
}

fn rebuilt(records: &[Record]) -> State {
    journal::read(journal_of(records).path()).unwrap()
}

fn rebuilt_of_a_search_that_builds(records: &[Record]) -> State {
    let settings = Settings {
        harness_build_command: Some("make harness".to_owned()),
        ..settings()
    };
    journal::read(journal_with(settings, records).path()).unwrap()
}

fn rebuilt_with_a_last_line_of(records: &[Record], last_line: &str) -> State {
    let run_directory = journal_of(records);
    OpenOptions::new()
        .append(true)
        .open(journal::path(run_directory.path()))
        .unwrap()
        .write_all(last_line.as_bytes())
        .unwrap();
    journal::read(run_directory.path()).unwrap()
}

fn four_jobs_of_which_two_finished() -> [Record; 6] {
    [
        procreation_started(1, 0),
        evaluation_started(0, "t1"),
        procreation_started(2, 0),
        evaluation_started(0, "t2"),
        procreation_finished(1, ProcreationOutcome::Created),
        evaluation_finished(0, "t1", EvaluationOutcome::Success),
    ]
}

#[test]
fn the_settings_of_the_run_record_come_back() {
    assert_eq!(rebuilt(&[]).settings, settings());
}

#[test]
fn a_journal_that_holds_no_job_gives_a_genealogy_of_the_progenitor_alone() {
    assert_eq!(rebuilt(&[]).genealogy, Genealogy::new());
}

#[test]
fn a_journal_of_a_search_that_builds_and_holds_no_build_gives_a_progenitor_awaiting_its_build() {
    assert_eq!(
        rebuilt_of_a_search_that_builds(&[]).genealogy,
        Genealogy::unbuilt()
    );
}

#[test]
fn a_journal_that_holds_the_build_of_its_progenitor_gives_a_created_progenitor() {
    assert_eq!(
        rebuilt_of_a_search_that_builds(&[Record::ProgenitorBuilt]).genealogy,
        Genealogy::new()
    );
}

#[test]
fn the_build_of_the_progenitor_is_not_counted_as_a_job() {
    assert_eq!(
        rebuilt_of_a_search_that_builds(&[Record::ProgenitorBuilt]).next_job,
        0
    );
}

#[test]
fn rebuilding_gives_the_genealogy_that_the_same_changes_give_directly() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(1, ProcreationOutcome::Created)
        .unwrap();
    genealogy.start_evaluation(1, "t2").unwrap();
    genealogy
        .end_evaluation(1, "t2", EvaluationOutcome::Success)
        .unwrap();
    genealogy.start_procreation(1).unwrap();
    genealogy.start_evaluation(0, "t1").unwrap();
    genealogy
        .end_evaluation(0, "t1", EvaluationOutcome::TimedOut)
        .unwrap();
    genealogy
        .end_procreation(2, ProcreationOutcome::Failed)
        .unwrap();
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, ProcreationOutcome::Created),
        evaluation_started(1, "t2"),
        evaluation_finished(1, "t2", EvaluationOutcome::Success),
        procreation_started(2, 1),
        evaluation_started(0, "t1"),
        evaluation_finished(0, "t1", EvaluationOutcome::TimedOut),
        procreation_finished(2, ProcreationOutcome::Failed),
    ]);
    assert_eq!(state.genealogy, genealogy);
}

#[test]
fn the_jobs_that_did_not_finish_come_back_in_the_order_they_started_with_their_numbers() {
    assert_eq!(
        rebuilt(&four_jobs_of_which_two_finished()).unfinished,
        [
            (
                2,
                Started::Procreation {
                    child: 2,
                    parent: 0
                }
            ),
            (
                3,
                Started::Evaluation {
                    harness: 0,
                    task: "t2".to_owned()
                }
            ),
        ]
    );
}

#[test]
fn a_journal_whose_jobs_all_finished_holds_no_unfinished_job() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, ProcreationOutcome::Created),
        evaluation_started(1, "t1"),
        evaluation_finished(1, "t1", EvaluationOutcome::Failure),
    ]);
    assert_eq!(state.unfinished, []);
}

#[test]
fn the_number_of_the_next_job_is_the_count_of_started_records() {
    assert_eq!(rebuilt(&four_jobs_of_which_two_finished()).next_job, 4);
}

#[test]
fn the_first_job_of_a_journal_that_holds_no_job_takes_number_0() {
    assert_eq!(rebuilt(&[]).next_job, 0);
}

#[test]
fn a_journal_that_holds_the_end_of_its_search_names_the_best_harness() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, ProcreationOutcome::Created),
        Record::SearchFinished { best_harness: 1 },
    ]);
    assert_eq!(state.best_harness, Some(1));
}

#[test]
fn a_journal_that_does_not_hold_the_end_of_its_search_names_no_best_harness() {
    assert_eq!(rebuilt(&[procreation_started(1, 0)]).best_harness, None);
}

#[test]
fn a_cut_short_last_line_is_ignored_when_the_journal_is_read() {
    let state = rebuilt_with_a_last_line_of(
        &[procreation_started(1, 0)],
        "{\"record\":\"procreation_finished\",\"chi",
    );
    assert_eq!(state, rebuilt(&[procreation_started(1, 0)]));
}

#[test]
fn a_whole_record_with_no_newline_at_its_end_is_ignored_as_cut_short() {
    let state =
        rebuilt_with_a_last_line_of(&[], "{\"record\":\"search_finished\",\"best_harness\":0}");
    assert_eq!(state.best_harness, None);
}

#[test]
fn reading_a_journal_takes_no_lock() {
    let run_directory = journal_of(&[]);
    let _writer = Journal::open(run_directory.path()).unwrap();
    assert!(journal::read(run_directory.path()).is_ok());
}
