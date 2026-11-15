use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::job::JobError;
use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

struct Searches {
    directory: TempDir,
    unbroken: Found,
    stopped: Result<Found, SearchError>,
    resumed: Result<Found, SearchError>,
}

fn settings(directory: &Path) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        tasks: (1..=5).map(|number| format!("t{number}")).collect(),
        expander: test_inputs::sample("integration", "expand-solve-one-more-or-exit-2.sh"),
        evaluator: test_inputs::sample("integration", "evaluate-listed-or-exit-2.sh"),
        budget: 12,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_expansions: 3,
        workers: 1,
        expansion_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn stopped_at(job: &str, resumes_while_stopped: usize) -> Searches {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let marker = directory.path().join("stopped/exit-2-at");
    let run_directory = directory.path().join("stopped/run");
    fs::create_dir(directory.path().join("unbroken")).unwrap();
    fs::create_dir(directory.path().join("stopped")).unwrap();
    let unbroken = directory.path().join("unbroken/run");
    let unbroken = search::search(&settings, &unbroken, &stop, None).unwrap();
    fs::write(&marker, format!("{job}\n")).unwrap();
    let mut stopped = search::search(&settings, &run_directory, &stop, None);
    for _ in 0..resumes_while_stopped {
        stopped = search::resume(&run_directory, &stop, None);
    }
    fs::remove_file(&marker).unwrap();
    let resumed = search::resume(&run_directory, &stop, None);
    Searches {
        directory,
        unbroken,
        stopped,
        resumed,
    }
}

fn journal_of(searches: &Searches, search: &str) -> String {
    let run_directory = searches.directory.path().join(search).join("run");
    fs::read_to_string(journal::path(&run_directory)).unwrap()
}

#[test]
fn an_evaluator_that_exits_with_2_at_a_task_stops_the_search_before_its_end() {
    assert!(matches!(
        stopped_at("t3", 0).stopped,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

#[test]
fn a_search_an_evaluator_stopped_continues_to_the_journal_an_unbroken_search_writes() {
    let searches = stopped_at("t3", 0);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn an_expander_that_exits_with_2_at_an_initial_expansion_stops_the_search_before_its_end() {
    assert!(matches!(
        stopped_at("expansion-2", 0).stopped,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

#[test]
fn a_search_stopped_during_its_initial_expansions_continues_to_the_same_journal_too() {
    let searches = stopped_at("expansion-2", 0);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn a_search_resumed_while_its_program_still_fails_stops_again() {
    assert!(matches!(
        stopped_at("t3", 1).stopped,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

#[test]
fn a_search_that_stopped_again_continues_to_the_same_journal_once_resumed_once_more() {
    let searches = stopped_at("t3", 1);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn a_resumed_search_returns_the_harness_an_unbroken_search_returns() {
    let searches = stopped_at("t3", 0);
    assert_eq!(
        searches.resumed.unwrap().best_harness,
        searches.unbroken.best_harness
    );
}

#[test]
fn a_resumed_search_returns_the_directory_of_the_best_harness_in_its_run_directory() {
    let searches = stopped_at("t3", 0);
    let best_harness = searches.unbroken.best_harness.to_string();
    assert_eq!(
        searches.resumed.unwrap().directory,
        searches
            .directory
            .path()
            .join("stopped/run/harnesses")
            .join(best_harness)
    );
}

#[test]
fn a_resumed_search_copies_only_the_records_it_writes_to_its_writer() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    let journal = journal::path(&run_directory);
    fs::write(directory.path().join("exit-2-at"), "t3\n").unwrap();
    let _stopped = search::search(&settings, &run_directory, &stop, None);
    fs::remove_file(directory.path().join("exit-2-at")).unwrap();
    let mut written = fs::read(&journal).unwrap();
    search::resume(&run_directory, &stop, Some(&mut written)).unwrap();
    assert_eq!(written, fs::read(&journal).unwrap());
}

#[test]
fn a_journal_cut_before_the_finished_record_of_a_job_in_place_continues_to_the_same_journal() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    let journal = journal::path(&run_directory);
    search::search(&settings, &run_directory, &stop, None).unwrap();
    let whole = fs::read_to_string(&journal).unwrap();
    let kept = whole.lines().count() - 2;
    let cut: String = whole.split_inclusive('\n').take(kept).collect();
    fs::write(&journal, cut).unwrap();
    search::resume(&run_directory, &stop, None).unwrap();
    assert_eq!(fs::read_to_string(&journal).unwrap(), whole);
}
