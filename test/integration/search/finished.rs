use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::journal::{self, Record, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(
                directory,
                &["t1", "t2", "t3", "t4", "t5"],
            ))
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
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
    settings.benchmarks[0].tasks = vec!["t1".to_owned()];
    settings.genealogy_growth = 0.0;
    settings.initial_procreations = 0;
    settings.budget = 10;
}

fn searched(change: impl FnOnce(&mut Settings)) -> (TempDir, Result<Found, SearchError>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    change(&mut settings);
    let run_directory = directory.path().join("run");
    let found = search::search(
        &settings,
        &run_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    (directory, found)
}

fn said_over_one_task() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    over_one_task(&mut settings);
    let run_directory = directory.path().join("run");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    search::search(&settings, &run_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    (directory, said.lines().map(str::to_owned).collect())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

fn started(journal: &str, action: &str) -> usize {
    journal
        .matches(&format!("\"record\":\"{action}_started\""))
        .count()
}

#[test]
fn with_one_worker_two_searches_with_the_same_settings_write_identical_journals() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    search::search(&settings, &first, &AtomicBool::new(false), &mut io::sink()).unwrap();
    search::search(&settings, &second, &AtomicBool::new(false), &mut io::sink()).unwrap();
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
    search::search(
        &settings,
        &run_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
    .unwrap();
    let first: Record =
        serde_json::from_str(journal_of(&directory).lines().next().unwrap()).unwrap();
    assert_eq!(
        first,
        Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        }
    );
}

#[test]
fn a_search_keeps_a_copy_of_the_tasks_of_its_benchmark_in_its_run_directory() {
    let (directory, _found) = searched(over_one_task);
    assert_eq!(
        fs::read_to_string(
            directory
                .path()
                .join("run/benchmarks/benchmark/t1/statement")
        )
        .unwrap(),
        "Finish the work of t1.\n"
    );
}

#[test]
fn a_search_copies_no_task_of_its_benchmark_that_is_not_one_of_its_own() {
    let (directory, _found) = searched(over_one_task);
    assert!(
        !directory
            .path()
            .join("run/benchmarks/benchmark/t2")
            .exists()
    );
}

#[test]
fn a_search_whose_benchmark_lacks_one_of_its_tasks_cannot_make_its_run_directory() {
    let (_directory, found) =
        searched(|settings| settings.benchmarks[0].tasks.push("missing".to_owned()));
    assert!(matches!(found, Err(SearchError::Io(_))));
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
fn a_search_says_first_that_it_makes_its_run_directory_with_a_copy_of_the_progenitor() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said[0],
        format!(
            "cladenc: making the run directory {} with a copy of the progenitor {}",
            directory.path().join("run").display(),
            directory.path().join("progenitor").display()
        )
    );
}

#[test]
fn a_search_says_next_that_it_copies_the_tasks_of_its_benchmark() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said[1],
        format!(
            "cladenc: copying the 1 task of the benchmark {} into the run directory",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_search_says_each_action_as_it_starts() {
    let (_directory, said) = said_over_one_task();
    assert_eq!(
        said[2],
        "cladenc: started the procreation of harness 1 by harness 0"
    );
}

#[test]
fn a_search_says_as_many_starts_as_its_journal_records() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said.iter()
            .filter(|line| line.starts_with("cladenc: started the "))
            .count(),
        journal_of(&directory).matches("_started\"").count()
    );
}

#[test]
fn a_search_says_the_number_of_each_evaluation_it_starts_and_its_budget() {
    let (_directory, said) = said_over_one_task();
    assert!(said.iter().any(|line| {
        line.starts_with("cladenc: started the evaluation of harness ")
            && line.ends_with(" on task benchmark/t1, evaluation 2 of 10")
    }));
}

#[test]
fn a_search_says_how_a_procreation_ended_and_where_its_child_is_kept() {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 created harness 1, kept in {}",
        directory.path().join("run/harnesses/1").display()
    )));
}

#[test]
fn a_search_says_how_an_evaluation_ended_and_where_what_it_left_is_kept() {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: the evaluation of harness 1 on task benchmark/t1 succeeded: what it left is kept in {}",
        directory.path().join("run/evaluations/1/benchmark/t1").display()
    )));
}

#[test]
fn a_search_says_last_that_it_finished_and_which_harness_is_its_best() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said.last().unwrap(),
        &format!(
            "cladenc: the search finished with 2 evaluations started and 2 harnesses: its best \
             harness is harness 1, kept in {}",
            directory.path().join("run/harnesses/1").display()
        )
    );
}

#[test]
fn a_search_copies_no_record_of_its_journal_to_its_writer() {
    let (_directory, said) = said_over_one_task();
    assert!(!said.iter().any(|line| line.contains("\"record\"")));
}

#[test]
fn a_search_cannot_start_in_a_run_directory_that_exists() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let found = search::search(
        &settings,
        directory.path(),
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    assert!(matches!(found, Err(SearchError::Io(_))));
}

#[test]
fn a_search_whose_run_record_cannot_be_written_stops_before_any_action() {
    let (_directory, found) = searched(|settings| {
        let unwritable = OsStr::from_bytes(&[b'p', 0xff]);
        settings.progenitor =
            test_inputs::progenitor(&settings.progenitor.with_file_name(unwritable));
    });
    assert!(matches!(found, Err(SearchError::Journal(_))));
}
