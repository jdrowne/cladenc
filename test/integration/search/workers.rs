use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::journal::{self, Record, Settings};
use cladenc::search;

fn records(change: impl FnOnce(&mut Settings)) -> Vec<Record> {
    let directory = test_inputs::temporary_directory();
    let mut settings = Settings {
        progenitor: test_inputs::progenitor(directory.path()),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmark: test_inputs::benchmark(directory.path(), &["t1", "t2", "t3", "t4", "t5"]),
        tasks: (1..=5).map(|number| format!("t{number}")).collect(),
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        evaluator: test_inputs::sample("integration", "evaluate-listed.sh"),
        budget: 12,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 3,
        workers: 3,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    };
    change(&mut settings);
    let run_directory = directory.path().join("run");
    search::search(&settings, &run_directory, &AtomicBool::new(false), None).unwrap();
    fs::read_to_string(journal::path(&run_directory))
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
            Record::ProcreationStarted { .. } | Record::EvaluationStarted { .. } => {
                running += 1;
                most = most.max(running);
            }
            Record::ProcreationFinished { .. } | Record::EvaluationFinished { .. } => running -= 1,
            Record::Run { .. } | Record::ProgenitorBuilt | Record::SearchFinished { .. } => {}
        }
    }
    most
}

fn count(records: &[Record], counted: impl Fn(&Record) -> bool) -> usize {
    records.iter().filter(|record| counted(record)).count()
}

#[test]
fn with_three_workers_the_journal_holds_three_jobs_started_and_not_finished_at_most() {
    assert_eq!(most_running_at_once(&records(|_| {})), 3);
}

#[test]
fn with_three_workers_the_journal_still_holds_exactly_as_many_evaluations_as_the_budget() {
    let records = records(|_| {});
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::EvaluationStarted { .. }
        )),
        12
    );
}

#[test]
fn with_a_growth_of_0_and_three_workers_exactly_one_procreation_starts() {
    let records = records(|settings| {
        settings.genealogy_growth = 0.0;
        settings.initial_procreations = 0;
    });
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::ProcreationStarted { .. }
        )),
        1
    );
}
