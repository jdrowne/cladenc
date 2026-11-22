use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

type Evolved = (TempDir, Result<Found, DirectedEvolutionError>);

const COUNT_THE_BUILDS: &str = "echo built >> built";
const BUILD_ONLY_WHAT_SOLVES_NOTHING: &str = "[ ! -e solved ]";
const BUILD_UNLESS_TOLD_NOT_TO: &str = "[ ! -e do-not-build ] && echo built >> built";

fn settings(directory: &Path, build_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: Some(build_command.to_owned()),
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved_with_its_stop_flag_at(build_command: &str, stop: bool) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), build_command);
    let outputs_directory = directory.path().join("outputs");
    let found = directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(stop),
        &mut io::sink(),
    );
    (directory, found)
}

fn evolved(build_command: &str) -> Evolved {
    evolved_with_its_stop_flag_at(build_command, false)
}

fn resumed_after_its_primogenitor_did_not_build() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), BUILD_UNLESS_TOLD_NOT_TO);
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    fs::write(settings.primogenitor.join("do-not-build"), "").unwrap();
    let _stopped = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    fs::remove_file(outputs_directory.join("harnesses/0/do-not-build")).unwrap();
    let resumed = directed_evolution::resume(&outputs_directory, &stop, &mut io::sink());
    (directory, resumed)
}

fn resumed_after_an_attempt_stopped_it() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), COUNT_THE_BUILDS);
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_on(directory.path(), "t1");
    let _stopped = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t1");
    let resumed = directed_evolution::resume(&outputs_directory, &stop, &mut io::sink());
    (directory, resumed)
}

fn stopped_while_its_primogenitor_is_built() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "hostname > id; sleep 300");
    let outputs_directory = directory.path().join("outputs");
    let id_file = outputs_directory.join("harnesses/0/id");
    let stop = AtomicBool::new(false);
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink())
    });
    (directory, found)
}

fn said_by_a_directed_evolution_built_with(build_command: &str) -> (TempDir, String) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), build_command);
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let _found = directed_evolution::start(&settings, &outputs_directory, &stop, &mut written);
    (directory, String::from_utf8(written).unwrap())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap()
}

fn builds_of(directory: &TempDir, harness: u64) -> String {
    let built = format!("outputs/harnesses/{harness}/built");
    fs::read_to_string(directory.path().join(built)).unwrap()
}

#[test]
fn the_build_of_the_primogenitor_is_recorded_right_after_the_run_record() {
    let (directory, _found) = evolved(COUNT_THE_BUILDS);
    assert_eq!(
        journal_of(&directory).lines().nth(1),
        Some(r#"{"record":"primogenitor_built"}"#)
    );
}

#[test]
fn the_primogenitor_of_a_directed_evolution_is_built_once() {
    let (directory, _found) = evolved(COUNT_THE_BUILDS);
    assert_eq!(builds_of(&directory, 0), "built\n");
}

#[test]
fn a_child_is_built_once_more_than_the_progenitor_it_was_copied_from() {
    let (directory, _found) = evolved(COUNT_THE_BUILDS);
    assert_eq!(builds_of(&directory, 1), "built\nbuilt\n");
}

#[test]
fn a_directed_evolution_that_builds_its_harnesses_returns_the_harness_the_samples_make_best() {
    let (_directory, found) = evolved(COUNT_THE_BUILDS);
    assert_eq!(found.unwrap().best_harness, 1);
}

#[test]
fn a_child_that_does_not_build_is_recorded_as_build_failed() {
    let (directory, _found) = evolved(BUILD_ONLY_WHAT_SOLVES_NOTHING);
    assert!(
        journal_of(&directory)
            .contains(r#"{"record":"procreation_finished","child":1,"outcome":"build_failed","#)
    );
}

#[test]
fn a_directed_evolution_goes_on_to_its_end_past_a_child_that_did_not_build() {
    let (_directory, found) = evolved(BUILD_ONLY_WHAT_SOLVES_NOTHING);
    assert_eq!(found.unwrap().best_harness, 0);
}

#[test]
fn a_primogenitor_that_does_not_build_stops_the_directed_evolution_with_the_status_of_its_build() {
    let (_directory, found) = evolved("exit 3");
    assert!(matches!(
        found,
        Err(DirectedEvolutionError::Build(ContainerError::Exit(3)))
    ));
}

#[test]
fn a_directed_evolution_whose_primogenitor_did_not_build_holds_only_its_run_record() {
    let (directory, _found) = evolved("exit 3");
    assert_eq!(journal_of(&directory).lines().count(), 1);
}

#[test]
fn a_directed_evolution_whose_primogenitor_did_not_build_builds_it_when_resumed() {
    let (directory, _resumed) = resumed_after_its_primogenitor_did_not_build();
    assert_eq!(
        journal_of(&directory).lines().nth(1),
        Some(r#"{"record":"primogenitor_built"}"#)
    );
}

#[test]
fn a_directed_evolution_whose_primogenitor_was_built_on_resuming_goes_on_to_its_end() {
    let (_directory, resumed) = resumed_after_its_primogenitor_did_not_build();
    assert_eq!(resumed.unwrap().best_harness, 1);
}

#[test]
fn a_resumed_directed_evolution_whose_primogenitor_was_built_does_not_build_it_again() {
    let (directory, _resumed) = resumed_after_an_attempt_stopped_it();
    assert_eq!(builds_of(&directory, 0), "built\n");
}

#[test]
fn a_resumed_directed_evolution_whose_primogenitor_was_built_goes_on_to_its_end() {
    let (_directory, resumed) = resumed_after_an_attempt_stopped_it();
    assert_eq!(resumed.unwrap().best_harness, 1);
}

#[test]
fn a_directed_evolution_asked_to_stop_while_its_primogenitor_is_built_stops() {
    let (_directory, found) = stopped_while_its_primogenitor_is_built();
    assert!(matches!(found, Err(DirectedEvolutionError::Asked)));
}

#[test]
fn a_directed_evolution_asked_to_stop_while_its_primogenitor_is_built_leaves_no_container_behind() {
    let (directory, _found) = stopped_while_its_primogenitor_is_built();
    assert!(test_inputs::container_is_gone(
        &directory.path().join("outputs/harnesses/0/id")
    ));
}

#[test]
fn a_directed_evolution_asked_to_stop_before_it_starts_was_never_started() {
    let (_directory, found) = evolved_with_its_stop_flag_at(COUNT_THE_BUILDS, true);
    assert!(matches!(found, Err(DirectedEvolutionError::NotStarted)));
}

#[test]
fn a_directed_evolution_asked_to_stop_before_it_starts_does_not_build_its_primogenitor() {
    let (directory, _found) = evolved_with_its_stop_flag_at(COUNT_THE_BUILDS, true);
    assert!(!directory.path().join("outputs/builds").exists());
}

#[test]
fn a_directed_evolution_says_that_it_builds_its_primogenitor_and_then_that_it_is_built() {
    let (_directory, said) = said_by_a_directed_evolution_built_with(COUNT_THE_BUILDS);
    assert!(
        said.contains("cladenc: building the primogenitor\ncladenc: the primogenitor is built\n")
    );
}

#[test]
fn a_directed_evolution_whose_primogenitor_does_not_build_says_where_what_the_build_printed_is_kept()
 {
    let (directory, said) = said_by_a_directed_evolution_built_with("exit 3");
    assert!(said.ends_with(&format!(
        "cladenc: the build of the primogenitor exited with status 3: what it printed is kept in \
         {}\n",
        directory.path().join("outputs/builds/0").display()
    )));
}

#[test]
fn a_directed_evolution_says_of_a_child_that_did_not_build_where_what_its_build_printed_is_kept() {
    let (directory, said) = said_by_a_directed_evolution_built_with(BUILD_ONLY_WHAT_SOLVES_NOTHING);
    assert!(said.contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 made a harness that did not build: \
         what its build printed is kept in {}\n",
        directory.path().join("outputs/builds/1").display()
    )));
}
