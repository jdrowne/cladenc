use std::ffi::OsStr;
use std::fs;
use std::io;
use std::iter;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Record, Settings};
use tempfile::TempDir;

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
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        max_procreations: 1,
        genealogy_growth: 0.6,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        attempt_timeout: 30,
        holdout_percentage: 20.0,
        holdout_tasks: vec!["benchmark/t1".to_owned()],
        seed: 7,
    }
}

const KEEP_THE_TASKS_FOUND: &str = "if [ -d /var/log/cladenc/attempts ]; then ls \
                                    /var/log/cladenc/attempts/benchmark > tasks-found; fi";

const FAIL_TO_PROCREATE: &str = "[ ! -d /var/log/cladenc/attempts ]";

fn over_one_task(settings: &mut Settings) {
    settings.benchmarks[0].tasks = vec!["t1".to_owned()];
    settings.genealogy_growth = 0.0;
}

fn over_two_benchmarks(settings: &mut Settings) {
    let directory = settings.primogenitor.parent().unwrap();
    let refactoring = test_inputs::benchmark_named(directory, "refactoring", &["t1", "t2"]);
    settings
        .benchmarks
        .push(Benchmark::read(&refactoring).unwrap());
    settings.genealogy_growth = 0.0;
}

fn evolved(change: impl FnOnce(&mut Settings)) -> (TempDir, Result<Found, DirectedEvolutionError>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    change(&mut settings);
    let outputs_directory = directory.path().join("outputs");
    let found = directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    (directory, found)
}

fn said_over_one_task() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    over_one_task(&mut settings);
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    (directory, said.lines().map(str::to_owned).collect())
}

fn journal_of(directory: &TempDir) -> String {
    fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap()
}

fn started(journal: &str, action: &str) -> usize {
    journal
        .matches(&format!("\"record\":\"{action}_started\""))
        .count()
}

fn starts(directory: &TempDir) -> Vec<String> {
    journal_of(directory)
        .lines()
        .filter_map(|line| match serde_json::from_str::<Record>(line).unwrap() {
            Record::AttemptStarted { harness, .. } => Some(format!("attempt of {harness}")),
            Record::ProcreationStarted { progenitor, .. } => {
                Some(format!("procreation by {progenitor}"))
            }
            _ => None,
        })
        .collect()
}

fn kinds_of_starts(directory: &TempDir) -> Vec<String> {
    starts(directory)
        .into_iter()
        .map(|start| start.split(" by ").next().unwrap_or_default().to_owned())
        .collect()
}

fn attempts_started(directory: &TempDir) -> Vec<(u64, String)> {
    let mut attempts: Vec<(u64, String)> = journal_of(directory)
        .lines()
        .filter_map(|line| match serde_json::from_str::<Record>(line).unwrap() {
            Record::AttemptStarted { harness, task } => Some((harness, task)),
            _ => None,
        })
        .collect();
    attempts.sort();
    attempts
}

fn every_task_for_each_of(harnesses: &[u64]) -> Vec<(u64, String)> {
    harnesses
        .iter()
        .flat_map(|harness| (1..=5).map(move |task| (*harness, format!("benchmark/t{task}"))))
        .collect()
}

fn attempts_of_the_primogenitor_and_then(tasks: usize, then: &[&str]) -> Vec<String> {
    iter::repeat_n("attempt of 0", tasks)
        .chain(then.iter().copied())
        .map(str::to_owned)
        .collect()
}

#[test]
fn one_container_at_a_time_two_directed_evolutions_of_the_same_settings_write_the_same_journal() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    over_one_task(&mut settings);
    settings.max_procreations = 3;
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    directed_evolution::start(&settings, &first, &AtomicBool::new(false), &mut io::sink()).unwrap();
    directed_evolution::start(&settings, &second, &AtomicBool::new(false), &mut io::sink())
        .unwrap();
    assert_eq!(
        test_inputs::without_seconds(&fs::read_to_string(journal::path(&first)).unwrap()),
        test_inputs::without_seconds(&fs::read_to_string(journal::path(&second)).unwrap())
    );
}

#[test]
fn the_first_record_of_a_directed_evolution_is_the_run_record_with_its_settings() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    directed_evolution::start(
        &settings,
        &outputs_directory,
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
fn a_directed_evolution_keeps_a_copy_of_the_tasks_of_its_benchmark_in_its_outputs_directory() {
    let (directory, _found) = evolved(over_one_task);
    assert_eq!(
        fs::read_to_string(
            directory
                .path()
                .join("outputs/benchmarks/benchmark/t1/specification")
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
            .join("outputs/benchmarks/benchmark/t2")
            .exists()
    );
}

#[test]
fn a_directed_evolution_whose_benchmark_lacks_one_of_its_tasks_cannot_make_its_outputs_directory() {
    let (_directory, found) =
        evolved(|settings| settings.benchmarks[0].tasks.push("missing".to_owned()));
    assert!(matches!(found, Err(DirectedEvolutionError::Io(_))));
}

#[test]
fn with_a_growth_above_0_a_directed_evolution_starts_with_an_attempt_of_the_primogenitor() {
    let (directory, _found) = evolved(|settings| settings.benchmarks[0].tasks.truncate(1));
    assert!(
        journal_of(&directory)
            .lines()
            .nth(1)
            .unwrap()
            .starts_with(r#"{"record":"attempt_started","harness":0,"#)
    );
}

#[test]
fn with_a_growth_above_0_the_primogenitor_procreates_once_it_has_attempted_every_task() {
    let (directory, _found) = evolved(|settings| settings.max_procreations = 2);
    assert_eq!(
        starts(&directory)[..7],
        attempts_of_the_primogenitor_and_then(5, &["procreation by 0", "procreation by 0"])
    );
}

#[test]
fn with_a_growth_of_0_each_child_is_benchmarked_in_full_before_the_next_procreation_starts() {
    let (directory, _found) = evolved(|settings| {
        settings.genealogy_growth = 0.0;
        settings.max_procreations = 2;
        settings.benchmarks[0].tasks.truncate(2);
    });
    assert_eq!(
        kinds_of_starts(&directory),
        [
            "attempt of 0",
            "attempt of 0",
            "procreation",
            "attempt of 1",
            "attempt of 1",
            "procreation",
            "attempt of 2",
            "attempt of 2",
        ]
    );
}

#[test]
fn given_two_benchmarks_the_primogenitor_procreates_once_benchmarked_on_both() {
    let (directory, _found) = evolved(over_two_benchmarks);
    assert_eq!(
        starts(&directory)[..9],
        attempts_of_the_primogenitor_and_then(7, &["procreation by 0", "attempt of 1"])
    );
}

#[test]
fn a_progenitor_finds_every_task_that_is_not_held_out_in_the_directory_of_its_attempts() {
    let (directory, _found) =
        evolved(|settings| settings.harness_run_command = KEEP_THE_TASKS_FOUND.to_owned());
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/1/tasks-found")).unwrap(),
        "t2\nt3\nt4\nt5\n"
    );
}

#[test]
fn a_directed_evolution_starts_exactly_as_many_procreations_as_the_most_it_was_given() {
    let (directory, _found) = evolved(|settings| {
        over_one_task(settings);
        settings.max_procreations = 3;
    });
    assert_eq!(started(&journal_of(&directory), "procreation"), 3);
}

#[test]
fn a_procreation_that_fails_is_one_of_the_most_a_directed_evolution_starts() {
    let (directory, _found) = evolved(|settings| {
        over_one_task(settings);
        settings.max_procreations = 2;
        settings.harness_run_command = FAIL_TO_PROCREATE.to_owned();
    });
    assert_eq!(started(&journal_of(&directory), "procreation"), 2);
}

#[test]
fn every_created_harness_of_a_directed_evolution_attempts_every_task_once() {
    let (directory, _found) = evolved(|_| {});
    assert_eq!(
        attempts_started(&directory),
        every_task_for_each_of(&[0, 1])
    );
}

#[test]
fn a_child_that_completes_none_of_its_tasks_still_attempts_every_task() {
    let (directory, _found) =
        evolved(|settings| settings.harness_run_command = KEEP_THE_TASKS_FOUND.to_owned());
    assert_eq!(
        attempts_started(&directory),
        every_task_for_each_of(&[0, 1])
    );
}

#[test]
fn a_directed_evolution_over_one_task_ends_once_every_created_harness_has_been_given_it() {
    let (directory, _found) = evolved(over_one_task);
    assert_eq!(started(&journal_of(&directory), "attempt"), 2);
}

#[test]
fn a_directed_evolution_ends_its_journal_with_its_finished_record() {
    let (directory, _found) = evolved(over_one_task);
    assert_eq!(
        journal_of(&directory).lines().last().unwrap(),
        r#"{"record":"directed_evolution_finished","best_harness":1}"#
    );
}

#[test]
fn the_end_of_each_attempt_and_of_each_procreation_is_recorded_with_the_seconds_it_took() {
    let (directory, _found) = evolved(over_one_task);
    let seconds: Vec<f64> = journal_of(&directory)
        .lines()
        .filter_map(|line| match serde_json::from_str::<Record>(line).unwrap() {
            Record::AttemptFinished { seconds, .. }
            | Record::ProcreationFinished { seconds, .. } => Some(seconds),
            _ => None,
        })
        .collect();
    assert_eq!(
        (seconds.len(), seconds.iter().all(|took| *took > 0.0)),
        (3, true)
    );
}

#[test]
fn the_harness_returned_is_the_one_the_samples_make_best() {
    let (_directory, found) = evolved(over_one_task);
    assert_eq!(found.unwrap().best_harness, 1);
}

#[test]
fn the_directory_returned_is_that_of_the_best_harness_in_the_outputs_directory() {
    let (directory, found) = evolved(over_one_task);
    assert_eq!(
        found.unwrap().directory,
        directory.path().join("outputs/harnesses/1")
    );
}

#[test]
fn a_directed_evolution_says_first_that_it_makes_its_outputs_directory_with_a_copy_of_the_primogenitor()
 {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said[0],
        format!(
            "cladenc: making the outputs directory {} with a copy of the primogenitor {}",
            directory.path().join("outputs").display(),
            directory.path().join("primogenitor").display()
        )
    );
}

#[test]
fn a_directed_evolution_says_next_that_it_copies_the_tasks_of_its_benchmark() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said[1],
        format!(
            "cladenc: copying the 1 task of the benchmark {} into the outputs directory",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn a_directed_evolution_says_each_action_as_it_starts() {
    let (_directory, said) = said_over_one_task();
    assert_eq!(
        said[5],
        "cladenc: started the procreation of harness 1 by harness 0, procreation 1 of 1"
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
fn a_directed_evolution_says_an_attempt_it_starts_with_no_number() {
    let (_directory, said) = said_over_one_task();
    assert_eq!(
        said[3],
        "cladenc: started the attempt of harness 0 on task benchmark/t1"
    );
}

#[test]
fn a_directed_evolution_says_how_a_procreation_ended_and_where_its_child_is_kept() {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 created harness 1, kept in {}",
        directory.path().join("outputs/harnesses/1").display()
    )));
}

#[test]
fn a_directed_evolution_says_whether_a_harness_completed_its_task_and_where_what_the_attempt_left_is_kept()
 {
    let (directory, said) = said_over_one_task();
    assert!(said.contains(&format!(
        "cladenc: harness 1 completed task benchmark/t1: what the attempt left \
         is kept in {}",
        directory.path().join("outputs/holdout/1/benchmark/t1").display()
    )));
}

#[test]
fn a_directed_evolution_says_last_that_it_finished_and_which_harness_is_its_best() {
    let (directory, said) = said_over_one_task();
    assert_eq!(
        said.last().unwrap(),
        &format!(
            "cladenc: the directed evolution finished with 2 attempts started and 2 \
             harnesses: its best harness is harness 1, kept in {}",
            directory.path().join("outputs/harnesses/1").display()
        )
    );
}

#[test]
fn a_directed_evolution_copies_no_record_of_its_journal_to_its_writer() {
    let (_directory, said) = said_over_one_task();
    assert!(!said.iter().any(|line| line.contains("\"record\"")));
}

#[test]
fn a_directed_evolution_cannot_start_in_an_outputs_directory_that_exists() {
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
        settings.primogenitor =
            test_inputs::primogenitor(&settings.primogenitor.with_file_name(unwritable));
    });
    assert!(matches!(found, Err(DirectedEvolutionError::Journal(_))));
}
