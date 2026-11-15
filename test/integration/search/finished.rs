use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::journal::{self, Record, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        tasks: (1..=5).map(|number| format!("t{number}")).collect(),
        docker_image: test_inputs::IMAGE.to_owned(),
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        evaluator: test_inputs::sample("integration", "evaluate-listed.sh"),
        budget: 6,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 2,
        workers: 1,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn over_one_task(settings: &mut Settings) {
    settings.tasks = vec!["t1".to_owned()];
    settings.genealogy_growth = 0.0;
    settings.initial_procreations = 0;
    settings.budget = 10;
}

fn searched(change: impl FnOnce(&mut Settings)) -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    change(&mut settings);
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(false), None);
    (directory, found)
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

fn started(journal: &str, job: &str) -> usize {
    journal
        .matches(&format!("\"record\":\"{job}_started\""))
        .count()
}

#[test]
fn with_one_worker_two_searches_with_the_same_settings_write_identical_journals() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    search::search(&settings, &first, &AtomicBool::new(false), None).unwrap();
    search::search(&settings, &second, &AtomicBool::new(false), None).unwrap();
    assert_eq!(
        fs::read(journal::path(&first)).unwrap(),
        fs::read(journal::path(&second)).unwrap()
    );
}

#[test]
fn the_first_record_of_a_search_is_the_run_record_with_its_settings() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let run_directory = directory.path().join("run");
    search::search(&settings, &run_directory, &AtomicBool::new(false), None).unwrap();
    let first: Record =
        serde_json::from_str(journal_of(&directory).lines().next().unwrap()).unwrap();
    assert_eq!(
        first,
        Record::Run {
            format: journal::FORMAT,
            settings
        }
    );
}

#[test]
fn the_initial_procreations_all_end_before_the_first_evaluation_starts() {
    let (directory, _found) = searched(|settings| {
        settings.initial_procreations = 3;
        settings.workers = 2;
    });
    let journal = journal_of(&directory);
    let before_the_first_evaluation = journal
        .split("\"record\":\"evaluation_started\"")
        .next()
        .unwrap();
    assert_eq!(
        before_the_first_evaluation
            .matches("\"record\":\"procreation_finished\"")
            .count(),
        3
    );
}

#[test]
fn a_search_that_spends_its_budget_starts_exactly_as_many_evaluations() {
    let (directory, _found) = searched(|_| {});
    assert_eq!(started(&journal_of(&directory), "evaluation"), 6);
}

#[test]
fn a_search_over_one_task_ends_once_every_created_harness_has_been_given_it() {
    let (directory, _found) = searched(over_one_task);
    assert_eq!(started(&journal_of(&directory), "evaluation"), 2);
}

#[test]
fn the_journal_of_a_search_that_ended_with_budget_left_over_ends_with_search_finished() {
    let (directory, _found) = searched(over_one_task);
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"search_finished","best_harness":1}"#
    );
}

#[test]
fn the_harness_returned_is_the_one_the_samples_make_best() {
    let (_directory, found) = searched(over_one_task);
    assert_eq!(found.unwrap().best_harness, 1);
}

#[test]
fn the_directory_returned_is_that_of_the_best_harness_in_the_run_directory() {
    let (directory, found) = searched(over_one_task);
    assert_eq!(
        found.unwrap().directory,
        directory.path().join("run/harnesses/1")
    );
}

#[test]
fn the_writer_receives_each_record_as_the_line_the_journal_holds() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let run_directory = directory.path().join("run");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    search::search(&settings, &run_directory, &stop, Some(&mut written)).unwrap();
    assert_eq!(written, fs::read(journal::path(&run_directory)).unwrap());
}

#[test]
fn a_search_cannot_start_in_a_run_directory_that_exists() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let found = search::search(&settings, directory.path(), &AtomicBool::new(false), None);
    assert!(matches!(found, Err(SearchError::Io(_))));
}

#[test]
fn a_search_whose_run_record_cannot_be_written_stops_before_any_job() {
    let (_directory, found) = searched(|settings| {
        settings.evaluator = PathBuf::from(OsStr::from_bytes(&[b'/', 0xff]));
    });
    assert!(matches!(found, Err(SearchError::Journal(_))));
}
