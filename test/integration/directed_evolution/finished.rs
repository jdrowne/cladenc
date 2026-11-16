use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Record, Settings};
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
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        assessment_timeout: 30,
        seed: 7,
    }
}

fn over_one_task(settings: &mut Settings) {
    settings.benchmarks[0].tasks = vec!["t1".to_owned()];
    settings.genealogy_growth = 0.0;
    settings.initial_procreations = 0;
    settings.budget = 10;
}

fn evolved(change: impl FnOnce(&mut Settings)) -> (TempDir, Result<Found, DirectedEvolutionError>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    change(&mut settings);
    let run_directory = directory.path().join("run");
    let found = directed_evolution::start(
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
    directed_evolution::start(&settings, &run_directory, &stop, &mut written).unwrap();
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
fn one_container_at_a_time_two_directed_evolutions_of_the_same_settings_write_the_same_journal() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    directed_evolution::start(&settings, &first, &AtomicBool::new(false), &mut io::sink()).unwrap();
    directed_evolution::start(&settings, &second, &AtomicBool::new(false), &mut io::sink())
        .unwrap();
    assert_eq!(
        fs::read(journal::path(&first)).unwrap(),
        fs::read(journal::path(&second)).unwrap()
    );
}

#[test]
fn the_first_record_of_a_directed_evolution_is_the_run_record_with_its_settings() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let run_directory = directory.path().join("run");
    directed_evolution::start(
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
fn a_directed_evolution_keeps_a_copy_of_the_tasks_of_its_benchmark_in_its_run_directory() {
    let (directory, _found) = evolved(over_one_task);
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
fn a_directed_evolution_copies_no_task_of_its_benchmark_that_is_not_one_of_its_own() {
    let (directory, _found) = evolved(over_one_task);
    assert!(
        !directory
            .path()
            .join("run/benchmarks/benchmark/t2")
            .exists()
    );
}

#[test]
fn a_directed_evolution_whose_benchmark_lacks_one_of_its_tasks_cannot_make_its_run_directory() {
    let (_directory, found) =
        evolved(|settings| settings.benchmarks[0].tasks.push("missing".to_owned()));
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}

#[test]
fn the_initial_procreations_all_end_before_the_first_assessment_starts() {
    let (directory, _found) = evolved(|settings| {
        settings.initial_procreations = 3;
        settings.max_concurrent_containers = 2;
    });
    let journal = journal_of(&directory);
    let before_the_first_assessment = journal
        .split("\"record\":\"assessment_started\"")
        .next()
        .unwrap();
    assert_eq!(
        before_the_first_assessment
            .matches("\"record\":\"procreation_finished\"")
            .count(),
        3
    );
}

#[test]
fn a_directed_evolution_that_spends_its_budget_starts_exactly_as_many_assessments() {
    let (directory, _found) = evolved(|_| {});
    assert_eq!(started(&journal_of(&directory), "assessment"), 6);
}

#[test]
fn a_directed_evolution_over_one_task_ends_once_every_created_harness_has_been_given_it() {
    let (directory, _found) = evolved(over_one_task);
    assert_eq!(started(&journal_of(&directory), "assessment"), 2);
}

#[test]
fn a_directed_evolution_that_ended_with_budget_left_ends_its_journal_with_its_finished_record() {
    let (directory, _found) = evolved(over_one_task);
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"directed_evolution_finished","best_harness":1}"#
    );
}

#[test]
fn the_harness_returned_is_the_one_the_samples_make_best() {
    let (_directory, found) = evolved(over_one_task);
    assert_eq!(found.unwrap().best_harness, 1);
}

#[test]
fn the_directory_returned_is_that_of_the_best_harness_in_the_run_directory() {
    let (directory, found) = evolved(over_one_task);
    assert_eq!(
        found.unwrap().directory,
        directory.path().join("run/harnesses/1")
    );
}

#[test]
fn a_directed_evolution_says_first_that_it_makes_its_run_directory_with_a_copy_of_the_progenitor() {
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
fn a_directed_evolution_says_next_that_it_copies_the_tasks_of_its_benchmark() {
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
fn a_directed_evolution_says_each_action_as_it_starts() {
    let (_directory, said) = said_over_one_task();
    assert_eq!(
        said[2],
        "cladenc: started the procreation of harness 1 by harness 0"
    );
}

#[test]
fn a_directed_evolution_says_as_many_starts_as_its_journal_records() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said.iter()
            .filter(|line| line.starts_with("cladenc: started the "))
            .count(),
        journal_of(&directory).matches("_started\"").count()
    );
}

#[test]
fn a_directed_evolution_says_the_number_of_each_assessment_it_starts_and_its_budget() {
    let (_directory, said) = said_over_one_task();
    assert!(said.iter().any(|line| {
        line.starts_with("cladenc: started the assessment of harness ")
            && line.ends_with(" on task benchmark/t1, assessment 2 of 10")
    }));
}

#[test]
fn a_directed_evolution_says_how_a_procreation_ended_and_where_its_child_is_kept() {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 created harness 1, kept in {}",
        directory.path().join("run/harnesses/1").display()
    )));
}

#[test]
fn a_directed_evolution_says_how_an_assessment_ended_and_where_what_it_left_is_kept() {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: the assessment of harness 1 on task benchmark/t1 succeeded: what it left is \
         kept in {}",
        directory.path().join("run/assessments/1/benchmark/t1").display()
    )));
}

#[test]
fn a_directed_evolution_says_last_that_it_finished_and_which_harness_is_its_best() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said.last().unwrap(),
        &format!(
            "cladenc: the directed evolution finished with 2 assessments started and 2 \
             harnesses: its best harness is harness 1, kept in {}",
            directory.path().join("run/harnesses/1").display()
        )
    );
}

#[test]
fn a_directed_evolution_copies_no_record_of_its_journal_to_its_writer() {
    let (_directory, said) = said_over_one_task();
    assert!(!said.iter().any(|line| line.contains("\"record\"")));
}

#[test]
fn a_directed_evolution_cannot_start_in_a_run_directory_that_exists() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let found = directed_evolution::start(
        &settings,
        directory.path(),
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}

#[test]
fn a_directed_evolution_whose_run_record_cannot_be_written_stops_before_any_action() {
    let (_directory, found) = evolved(|settings| {
        let unwritable = OsStr::from_bytes(&[b'p', 0xff]);
        settings.progenitor =
            test_inputs::progenitor(&settings.progenitor.with_file_name(unwritable));
    });
    assert!(matches!(found, Err(DirectedEvolutionError::Journal(_))));
}
