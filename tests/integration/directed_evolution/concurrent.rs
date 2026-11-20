use std::fs;
use std::io;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution;
use cladenc::journal::{self, Record, Settings};

fn records(change: impl FnOnce(&mut Settings)) -> Vec<Record> {
    let directory = test_inputs::temporary_directory();
    let mut settings = Settings {
        primogenitor: test_inputs::primogenitor(directory.path()),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(
                directory.path(),
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
        max_concurrent_containers: 3,
        procreation_timeout: 30,
        attempt_timeout: 30,
        holdout_percentage: 50.0,
        holdout_tasks: Vec::new(),
        holdout_corroboration_percentage: 50.0,
        corroboration_tasks: Vec::new(),
        seed: 7,
    };
    change(&mut settings);
    let outputs_directory = directory.path().join("outputs");
    directed_evolution::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    )
    .unwrap();
    fs::read_to_string(journal::path(&outputs_directory))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn most_running_at_once(records: &[Record]) -> usize {
    let mut running = 0_usize;
    let mut most = 0;
    for record in records {
        match record {
            Record::ProcreationStarted { .. }
            | Record::BaselineStarted { .. }
            | Record::AttemptStarted { .. } => {
                running += 1;
                most = most.max(running);
            }
            Record::ProcreationFinished { .. }
            | Record::BaselineFinished { .. }
            | Record::AttemptFinished { .. } => running -= 1,
            Record::Run { .. }
            | Record::PrimogenitorBuilt
            | Record::DirectedEvolutionFinished { .. } => {}
        }
    }
    most
}

fn most_running_as_a_procreation_starts(records: &[Record]) -> Option<usize> {
    let mut running = 0_usize;
    let mut most = None;
    for record in records {
        match record {
            Record::ProcreationStarted { .. } => {
                most = most.max(Some(running));
                running += 1;
            }
            Record::BaselineStarted { .. } | Record::AttemptStarted { .. } => running += 1,
            Record::ProcreationFinished { .. }
            | Record::BaselineFinished { .. }
            | Record::AttemptFinished { .. } => running -= 1,
            Record::Run { .. }
            | Record::PrimogenitorBuilt
            | Record::DirectedEvolutionFinished { .. } => {}
        }
    }
    most
}

fn one_child_at_a_time(settings: &mut Settings) {
    settings.genealogy_growth = 0.0;
    settings.max_procreations = 2;
}

fn count(records: &[Record], counted: impl Fn(&Record) -> bool) -> usize {
    records.iter().filter(|record| counted(record)).count()
}

fn fewest_tasks_a_progenitor_had_attempted(records: &[Record]) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .filter_map(|(place, record)| match record {
            Record::ProcreationStarted { progenitor, .. } => {
                Some(count(&records[..place], |earlier| {
                    matches!(
                        earlier,
                        Record::AttemptFinished { harness, .. } if harness == progenitor
                    )
                }))
            }
            _ => None,
        })
        .min()
}

#[test]
fn with_a_limit_of_three_the_journal_holds_three_actions_started_and_not_finished_at_most() {
    assert_eq!(most_running_at_once(&records(|_| {})), 3);
}

#[test]
fn with_a_limit_of_three_the_journal_still_holds_one_attempt_of_every_task_by_each_harness() {
    let records = records(|_| {});
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::AttemptStarted { .. }
        )),
        10
    );
}

#[test]
fn with_a_growth_of_0_and_a_limit_of_three_every_procreation_still_starts() {
    let records = records(one_child_at_a_time);
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::ProcreationStarted { .. }
        )),
        2
    );
}

#[test]
fn with_a_growth_of_0_and_a_limit_of_three_a_procreation_starts_only_when_nothing_else_runs() {
    let records = records(one_child_at_a_time);
    assert_eq!(most_running_as_a_procreation_starts(&records), Some(0));
}

#[test]
fn with_a_limit_of_two_no_procreation_starts_before_its_progenitor_is_benchmarked_in_full() {
    let records = records(|settings| settings.max_concurrent_containers = 2);
    assert_eq!(fewest_tasks_a_progenitor_had_attempted(&records), Some(5));
}
