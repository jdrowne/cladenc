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
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmark: test_inputs::benchmark(directory, &["t1", "t2", "t3", "t4", "t5"]),
        tasks: (1..=5).map(|number| format!("t{number}")).collect(),
        docker_image: test_inputs::IMAGE.to_owned(),
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        evaluator: test_inputs::sample("integration", "evaluate-listed-or-exit-2.sh"),
        budget: 12,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 3,
        workers: 1,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn stopped_at(solved: u64, task: &str, resumes_while_stopped: usize) -> Searches {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let marker = directory.path().join("stopped/exit-2-at");
    let run_directory = directory.path().join("stopped/run");
    fs::create_dir(directory.path().join("unbroken")).unwrap();
    fs::create_dir(directory.path().join("stopped")).unwrap();
    let unbroken = directory.path().join("unbroken/run");
    let unbroken = search::search(&settings, &unbroken, &stop, None).unwrap();
    test_inputs::end_with_125_at(&settings.progenitor, solved);
    fs::write(&marker, format!("{task}\n")).unwrap();
    let mut stopped = search::search(&settings, &run_directory, &stop, None);
    for _ in 0..resumes_while_stopped {
        stopped = search::resume(&run_directory, &stop, None);
    }
    test_inputs::end_with_125_no_more(&run_directory);
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
        stopped_at(99, "t3", 0).stopped,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

#[test]
fn a_search_an_evaluator_stopped_continues_to_the_journal_an_unbroken_search_writes() {
    let searches = stopped_at(99, "t3", 0);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn a_container_that_ends_with_status_125_at_an_initial_procreation_stops_the_search() {
    assert!(matches!(
        stopped_at(0, "none", 0).stopped,
        Err(SearchError::Job {
            error: JobError::Docker(_),
            ..
        })
    ));
}

#[test]
fn a_search_stopped_during_its_initial_procreations_continues_to_the_same_journal_too() {
    let searches = stopped_at(0, "none", 0);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn a_search_resumed_while_its_program_still_fails_stops_again() {
    assert!(matches!(
        stopped_at(99, "t3", 1).stopped,
        Err(SearchError::Job {
            error: JobError::Exit(2),
            ..
        })
    ));
}

#[test]
fn a_search_that_stopped_again_continues_to_the_same_journal_once_resumed_once_more() {
    let searches = stopped_at(99, "t3", 1);
    assert_eq!(
        journal_of(&searches, "stopped"),
        journal_of(&searches, "unbroken")
    );
}

#[test]
fn a_resumed_search_returns_the_harness_an_unbroken_search_returns() {
    let searches = stopped_at(99, "t3", 0);
    assert_eq!(
        searches.resumed.unwrap().best_harness,
        searches.unbroken.best_harness
    );
}

#[test]
fn a_resumed_search_returns_the_directory_of_the_best_harness_in_its_run_directory() {
    let searches = stopped_at(99, "t3", 0);
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

fn lines_of(written: Vec<u8>) -> Vec<String> {
    let said = String::from_utf8(written).unwrap();
    said.lines().map(str::to_owned).collect()
}

fn said_when_stopped_at_t3_and_when_resumed() -> (TempDir, Vec<String>, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    let mut stopped = Vec::new();
    let mut resumed = Vec::new();
    fs::write(directory.path().join("exit-2-at"), "t3\n").unwrap();
    let _stopped = search::search(&settings, &run_directory, &stop, Some(&mut stopped));
    fs::remove_file(directory.path().join("exit-2-at")).unwrap();
    search::resume(&run_directory, &stop, Some(&mut resumed)).unwrap();
    (directory, lines_of(stopped), lines_of(resumed))
}

#[test]
fn a_search_that_a_job_stops_says_what_became_of_the_job() {
    let (_directory, stopped, _resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(stopped.iter().any(|line| {
        line.starts_with("cladenc: the evaluation of harness ")
            && line.ends_with(" on task t3 exited with status 2")
    }));
}

#[test]
fn a_search_that_a_job_stops_says_last_that_it_is_stopping() {
    let (_directory, stopped, _resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert_eq!(
        stopped.last().unwrap(),
        "cladenc: stopping the search, with 0 jobs running"
    );
}

#[test]
fn a_resumed_search_says_first_how_far_the_search_had_got() {
    let (directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(resumed[0].starts_with(&format!(
        "cladenc: continuing the search in {}, which has started ",
        directory.path().join("run").display()
    )));
}

#[test]
fn a_resumed_search_says_that_it_starts_again_the_job_that_was_running() {
    let (_directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(
        resumed[1].starts_with("cladenc: started again the evaluation of harness ")
            && resumed[1].ends_with(" on task t3")
    );
}

#[test]
fn a_resumed_search_says_last_that_it_finished() {
    let (_directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(
        resumed
            .last()
            .unwrap()
            .starts_with("cladenc: the search finished with 12 evaluations started and ")
    );
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

#[test]
fn a_search_is_resumed_without_the_directory_its_benchmark_was_given_in() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let run_directory = directory.path().join("run");
    test_inputs::end_with_125_at(&settings.progenitor, 0);
    let _ = search::search(&settings, &run_directory, &stop, None);
    test_inputs::end_with_125_no_more(&run_directory);
    fs::remove_dir_all(&settings.benchmark).unwrap();
    assert!(search::resume(&run_directory, &stop, None).is_ok());
}
