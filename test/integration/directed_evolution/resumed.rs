use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings, Started};
use tempfile::TempDir;

struct DirectedEvolutions {
    directory: TempDir,
    unbroken: Found,
    stopped: Result<Found, DirectedEvolutionError>,
    resumed: Result<Found, DirectedEvolutionError>,
}

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
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
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        max_procreations: 2,
        genealogy_growth: 0.6,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        attempt_timeout: 30,
        holdout_percentage: 50.0,
        holdout_tasks: Vec::new(),
        seed: 7,
    }
}

fn stopped_at(solved: u64, task: Option<&str>, resumes_while_stopped: usize) -> DirectedEvolutions {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("stopped/outputs");
    fs::create_dir(directory.path().join("unbroken")).unwrap();
    fs::create_dir(directory.path().join("stopped")).unwrap();
    let unbroken = directory.path().join("unbroken/outputs");
    let unbroken = directed_evolution::start(&settings, &unbroken, &stop, &mut io::sink()).unwrap();
    test_inputs::end_with_125_at(&settings.primogenitor, solved);
    if let Some(task) = task {
        test_inputs::end_with_125_on(directory.path(), task);
    }
    let mut stopped =
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    for _ in 0..resumes_while_stopped {
        stopped = directed_evolution::resume(&outputs_directory, &stop, &mut io::sink());
    }
    test_inputs::end_with_125_no_more(&outputs_directory);
    if let Some(task) = task {
        test_inputs::end_with_125_on_no_more(&outputs_directory, task);
    }
    let resumed = directed_evolution::resume(&outputs_directory, &stop, &mut io::sink());
    DirectedEvolutions {
        directory,
        unbroken,
        stopped,
        resumed,
    }
}

fn journal_of(directed_evolutions: &DirectedEvolutions, directed_evolution: &str) -> String {
    let outputs_directory = directed_evolutions
        .directory
        .path()
        .join(directed_evolution)
        .join("outputs");
    fs::read_to_string(journal::path(&outputs_directory)).unwrap()
}

#[test]
fn a_container_that_ends_with_status_125_at_an_attempt_stops_it_before_its_end() {
    assert!(matches!(
        stopped_at(99, Some("t3"), 0).stopped,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { .. },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_an_attempt_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    let directed_evolutions = stopped_at(99, Some("t3"), 0);
    assert_eq!(
        journal_of(&directed_evolutions, "stopped"),
        journal_of(&directed_evolutions, "unbroken")
    );
}

#[test]
fn a_container_that_ends_with_status_125_at_a_procreation_stops_it_before_its_end() {
    assert!(matches!(
        stopped_at(0, None, 0).stopped,
        Err(DirectedEvolutionError::Action {
            action: Started::Procreation { .. },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_a_procreation_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    let directed_evolutions = stopped_at(0, None, 0);
    assert_eq!(
        journal_of(&directed_evolutions, "stopped"),
        journal_of(&directed_evolutions, "unbroken")
    );
}

#[test]
fn a_directed_evolution_resumed_while_docker_still_does_not_start_the_container_stops_again() {
    assert!(matches!(
        stopped_at(99, Some("t3"), 1).stopped,
        Err(DirectedEvolutionError::Action {
            action: Started::Attempt { .. },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_that_stopped_again_continues_to_the_same_journal_once_resumed_once_more() {
    let directed_evolutions = stopped_at(99, Some("t3"), 1);
    assert_eq!(
        journal_of(&directed_evolutions, "stopped"),
        journal_of(&directed_evolutions, "unbroken")
    );
}

#[test]
fn a_resumed_directed_evolution_returns_the_harness_an_unbroken_directed_evolution_returns() {
    let directed_evolutions = stopped_at(99, Some("t3"), 0);
    assert_eq!(
        directed_evolutions.resumed.unwrap().best_harness,
        directed_evolutions.unbroken.best_harness
    );
}

#[test]
fn a_resumed_directed_evolution_returns_the_directory_of_the_best_harness_in_its_outputs_directory()
{
    let directed_evolutions = stopped_at(99, Some("t3"), 0);
    let best_harness = directed_evolutions.unbroken.best_harness.to_string();
    assert_eq!(
        directed_evolutions.resumed.unwrap().directory,
        directed_evolutions
            .directory
            .path()
            .join("stopped/outputs/harnesses")
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
    let outputs_directory = directory.path().join("outputs");
    let mut stopped = Vec::new();
    let mut resumed = Vec::new();
    test_inputs::end_with_125_on(directory.path(), "t3");
    let _stopped = directed_evolution::start(&settings, &outputs_directory, &stop, &mut stopped);
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t3");
    directed_evolution::resume(&outputs_directory, &stop, &mut resumed).unwrap();
    (directory, lines_of(stopped), lines_of(resumed))
}

#[test]
fn a_directed_evolution_that_an_action_stops_says_what_became_of_the_action() {
    let (_directory, stopped, _resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(stopped.iter().any(|line| {
        line.starts_with("cladenc: the attempt of harness ")
            && line.contains(
                " on task benchmark/t3 could not be carried out: Docker did not start its \
                 container, and ",
            )
    }));
}

#[test]
fn a_directed_evolution_that_an_action_stops_says_last_that_it_is_stopping() {
    let (_directory, stopped, _resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert_eq!(
        stopped.last().unwrap(),
        "cladenc: stopping the directed evolution, with 0 containers running"
    );
}

#[test]
fn a_resumed_directed_evolution_says_first_how_far_the_directed_evolution_had_got() {
    let (directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(resumed[0].starts_with(&format!(
        "cladenc: continuing the directed evolution in {}, which has started ",
        directory.path().join("outputs").display()
    )));
}

#[test]
fn a_resumed_directed_evolution_says_that_it_starts_again_the_action_that_was_running() {
    let (_directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(resumed[1..2].iter().any(|line| {
        line.starts_with("cladenc: started again the attempt of harness ")
            && line.ends_with(" on task benchmark/t3")
    }));
}

#[test]
fn a_resumed_directed_evolution_says_last_that_it_finished() {
    let (_directory, _stopped, resumed) = said_when_stopped_at_t3_and_when_resumed();
    assert!(
        resumed
            .last()
            .unwrap()
            .starts_with("cladenc: the directed evolution finished with 15 attempts started and ")
    );
}

#[test]
fn a_journal_cut_before_the_finished_record_of_an_action_in_place_continues_to_the_same_journal() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    let journal = journal::path(&outputs_directory);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    let whole = fs::read_to_string(&journal).unwrap();
    let kept = whole.lines().count() - 2;
    let cut: String = whole.split_inclusive('\n').take(kept).collect();
    fs::write(&journal, cut).unwrap();
    directed_evolution::resume(&outputs_directory, &stop, &mut io::sink()).unwrap();
    assert_eq!(fs::read_to_string(&journal).unwrap(), whole);
}

#[test]
fn a_directed_evolution_is_resumed_without_the_directory_its_benchmark_was_given_in() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    test_inputs::end_with_125_at(&settings.primogenitor, 0);
    let _ = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_no_more(&outputs_directory);
    fs::remove_dir_all(&settings.benchmarks[0].directory).unwrap();
    assert!(directed_evolution::resume(&outputs_directory, &stop, &mut io::sink()).is_ok());
}
