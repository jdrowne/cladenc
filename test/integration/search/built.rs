use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::job::JobError;
use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;

type Searched = (TempDir, Result<Found, SearchError>);

const COUNT_THE_BUILDS: &str = "echo built >> built";
const BUILD_ONLY_WHAT_SOLVES_NOTHING: &str = "[ ! -e solved ]";
const BUILD_UNLESS_TOLD_NOT_TO: &str = "[ ! -e do-not-build ] && echo built >> built";

fn settings(directory: &Path, build_command: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmark: test_inputs::benchmark(directory, &["t1"]),
        tasks: vec!["t1".to_owned()],
        docker_image: test_inputs::IMAGE.to_owned(),
        harness_build_command: Some(build_command.to_owned()),
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        evaluator: test_inputs::sample("integration", "evaluate-listed-or-exit-2.sh"),
        budget: 10,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_procreations: 0,
        workers: 1,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn searched_with_its_stop_flag_at(build_command: &str, stop: bool) -> Searched {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), build_command);
    let run_directory = directory.path().join("run");
    let found = search::search(&settings, &run_directory, &AtomicBool::new(stop), None);
    (directory, found)
}

fn searched(build_command: &str) -> Searched {
    searched_with_its_stop_flag_at(build_command, false)
}

fn resumed_after_its_progenitor_did_not_build() -> Searched {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), BUILD_UNLESS_TOLD_NOT_TO);
    let run_directory = directory.path().join("run");
    let stop = AtomicBool::new(false);
    fs::write(settings.progenitor.join("do-not-build"), "").unwrap();
    let _stopped = search::search(&settings, &run_directory, &stop, None);
    fs::remove_file(run_directory.join("harnesses/0/do-not-build")).unwrap();
    let resumed = search::resume(&run_directory, &stop, None);
    (directory, resumed)
}

fn resumed_after_an_evaluator_stopped_it() -> Searched {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), COUNT_THE_BUILDS);
    let run_directory = directory.path().join("run");
    let marker = directory.path().join("exit-2-at");
    let stop = AtomicBool::new(false);
    fs::write(&marker, "t1\n").unwrap();
    let _stopped = search::search(&settings, &run_directory, &stop, None);
    fs::remove_file(&marker).unwrap();
    let resumed = search::resume(&run_directory, &stop, None);
    (directory, resumed)
}

fn stopped_while_its_progenitor_is_built() -> Searched {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "hostname > id; sleep 300");
    let run_directory = directory.path().join("run");
    let id_file = run_directory.join("harnesses/0/id");
    let stop = AtomicBool::new(false);
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        search::search(&settings, &run_directory, &stop, None)
    });
    (directory, found)
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("run"))).unwrap()
}

fn builds_of(directory: &TempDir, harness: u64) -> String {
    let built = format!("run/harnesses/{harness}/built");
    fs::read_to_string(directory.path().join(built)).unwrap()
}

#[test]
fn the_build_of_the_progenitor_is_recorded_right_after_the_run_record() {
    let (directory, _found) = searched(COUNT_THE_BUILDS);
    assert_eq!(
        journal_of(&directory).lines().nth(1),
        Some(r#"{"record":"progenitor_built"}"#)
    );
}

#[test]
fn the_progenitor_of_a_search_is_built_once() {
    let (directory, _found) = searched(COUNT_THE_BUILDS);
    assert_eq!(builds_of(&directory, 0), "built\n");
}

#[test]
fn a_child_is_built_once_more_than_the_parent_it_was_copied_from() {
    let (directory, _found) = searched(COUNT_THE_BUILDS);
    assert_eq!(builds_of(&directory, 1), "built\nbuilt\n");
}

#[test]
fn a_search_that_builds_its_harnesses_returns_the_harness_the_samples_make_best() {
    let (_directory, found) = searched(COUNT_THE_BUILDS);
    assert_eq!(found.unwrap().best_harness, 1);
}

#[test]
fn a_child_that_does_not_build_is_recorded_as_build_failed() {
    let (directory, _found) = searched(BUILD_ONLY_WHAT_SOLVES_NOTHING);
    assert!(
        journal_of(&directory)
            .contains(r#"{"record":"procreation_finished","child":1,"outcome":"build_failed"}"#)
    );
}

#[test]
fn a_search_goes_on_to_its_end_past_a_child_that_did_not_build() {
    let (_directory, found) = searched(BUILD_ONLY_WHAT_SOLVES_NOTHING);
    assert_eq!(found.unwrap().best_harness, 0);
}

#[test]
fn a_progenitor_that_does_not_build_stops_the_search_with_the_status_of_its_build() {
    let (_directory, found) = searched("exit 3");
    assert!(matches!(found, Err(SearchError::Build(JobError::Exit(3)))));
}

#[test]
fn a_search_whose_progenitor_did_not_build_holds_only_its_run_record() {
    let (directory, _found) = searched("exit 3");
    assert_eq!(journal_of(&directory).lines().count(), 1);
}

#[test]
fn a_search_whose_progenitor_did_not_build_builds_it_when_resumed() {
    let (directory, _resumed) = resumed_after_its_progenitor_did_not_build();
    assert_eq!(
        journal_of(&directory).lines().nth(1),
        Some(r#"{"record":"progenitor_built"}"#)
    );
}

#[test]
fn a_search_whose_progenitor_was_built_on_resuming_goes_on_to_its_end() {
    let (_directory, resumed) = resumed_after_its_progenitor_did_not_build();
    assert_eq!(resumed.unwrap().best_harness, 1);
}

#[test]
fn a_resumed_search_whose_progenitor_was_built_does_not_build_it_again() {
    let (directory, _resumed) = resumed_after_an_evaluator_stopped_it();
    assert_eq!(builds_of(&directory, 0), "built\n");
}

#[test]
fn a_resumed_search_whose_progenitor_was_built_goes_on_to_its_end() {
    let (_directory, resumed) = resumed_after_an_evaluator_stopped_it();
    assert_eq!(resumed.unwrap().best_harness, 1);
}

#[test]
fn a_search_asked_to_stop_while_its_progenitor_is_built_stops() {
    let (_directory, found) = stopped_while_its_progenitor_is_built();
    assert!(matches!(found, Err(SearchError::Asked)));
}

#[test]
fn a_search_asked_to_stop_while_its_progenitor_is_built_leaves_no_container_behind() {
    let (directory, _found) = stopped_while_its_progenitor_is_built();
    assert!(test_inputs::container_is_gone(
        &directory.path().join("run/harnesses/0/id")
    ));
}

#[test]
fn a_search_asked_to_stop_before_it_starts_stops() {
    let (_directory, found) = searched_with_its_stop_flag_at(COUNT_THE_BUILDS, true);
    assert!(matches!(found, Err(SearchError::Asked)));
}

#[test]
fn a_search_asked_to_stop_before_it_starts_does_not_build_its_progenitor() {
    let (directory, _found) = searched_with_its_stop_flag_at(COUNT_THE_BUILDS, true);
    assert!(!directory.path().join("run/builds").exists());
}
