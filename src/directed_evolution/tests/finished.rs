use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use crate::benchmark::Benchmark;
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Record, Settings};
use crate::sandboxes::tests::double::{Answer, Double};

const DIVERSIFICATION: [&str; 2] = ["run", "run a harness"];

fn solving() -> Double {
    Double::default()
        .changing()
        .verifying(|trial| trial.starts_with("trial-1-"))
}

fn over_one_holdout_task(directory: &Memory, settings: &mut Settings) {
    given::over_one_task(directory, settings);
    settings.holdout_tasks = vec!["benchmark/t1".to_owned()];
}

fn with_three_diversifications(directory: &Memory, settings: &mut Settings) {
    over_one_holdout_task(directory, settings);
    settings.max_diversifications = 3;
}

fn with_two_diversifications(_: &Memory, settings: &mut Settings) {
    settings.max_diversifications = 2;
}

fn with_two_diversifications_over_one_task(directory: &Memory, settings: &mut Settings) {
    given::over_one_task(directory, settings);
    settings.max_diversifications = 2;
}

fn over_two_benchmarks(directory: &Memory, settings: &mut Settings) {
    let refactoring =
        test_inputs::benchmark_named_in(directory, directory.path(), "refactoring", &["t1", "t2"]);
    settings
        .benchmarks
        .push(Benchmark::read(directory, &refactoring).unwrap());
}

fn with_a_primogenitor_whose_path_cannot_be_written(directory: &Memory, settings: &mut Settings) {
    let unwritable = directory.path().join(OsStr::from_bytes(&[b'p', 0xff]));
    settings.primogenitor = test_inputs::primogenitor_in(directory, &unwritable);
}

fn said_over_one_task() -> Evolved {
    given::started(solving(), over_one_holdout_task)
}

fn kinds_of_starts(evolved: &Evolved) -> Vec<String> {
    evolved
        .starts()
        .into_iter()
        .filter(|start| !start.starts_with("baseline"))
        .map(|start| {
            if start.starts_with("diversification") {
                "diversification".to_owned()
            } else {
                start
            }
        })
        .collect()
}

fn started(evolved: &Evolved, action: &str) -> usize {
    evolved
        .starts()
        .iter()
        .filter(|start| start.starts_with(action))
        .count()
}

fn trials_started(evolved: &Evolved) -> Vec<(u64, String)> {
    let mut trials: Vec<(u64, String)> = evolved
        .records()
        .into_iter()
        .filter_map(|record| match record {
            Record::TrialStarted { harness, task } => Some((harness, task)),
            _ => None,
        })
        .collect();
    trials.sort();
    trials
}

#[test]
fn one_sandbox_at_a_time_two_directed_evolutions_of_the_same_settings_write_the_same_journal() {
    let [first, second] = [(), ()].map(|()| {
        let evolved = given::started(solving(), with_three_diversifications);
        test_inputs::without_seconds(&evolved.journal())
    });
    assert_eq!(first, second);
}

#[test]
fn the_first_record_of_a_directed_evolution_is_the_run_record_with_its_settings() {
    let mut started_with = None;
    let evolved = given::started(Double::default(), |_, settings| {
        started_with = Some(settings.clone());
    });
    assert_eq!(
        evolved.records().into_iter().next(),
        started_with.map(|settings| Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        })
    );
}

#[test]
fn a_directed_evolution_keeps_a_copy_of_the_tasks_of_its_benchmark_in_its_outputs_directory() {
    let evolved = given::started(Double::default(), given::over_one_task);
    let instruction = evolved
        .outputs()
        .join("benchmarks/benchmark/t1/instruction.md");
    assert_eq!(
        evolved.files().read_to_string(instruction).ok().as_deref(),
        Some("Finish the work of t1.\n")
    );
}

#[test]
fn a_directed_evolution_copies_no_task_of_its_benchmark_that_is_not_one_of_its_own() {
    let evolved = given::started(Double::default(), given::over_one_task);
    let other = evolved.outputs().join("benchmarks/benchmark/t2");
    assert!(!evolved.files().exists(other));
}

#[test]
fn each_child_is_benchmarked_in_full_before_the_next_diversification_starts() {
    let evolved = given::started(Double::default().changing(), with_two_diversifications);
    assert_eq!(
        kinds_of_starts(&evolved)[..8],
        [
            "trial of 0",
            "trial of 0",
            "diversification",
            "trial of 1",
            "trial of 1",
            "diversification",
            "trial of 2",
            "trial of 2",
        ]
    );
}

#[test]
fn given_two_benchmarks_the_primogenitor_is_cloned_once_benchmarked_on_both() {
    let evolved = given::started(Double::default().changing(), over_two_benchmarks);
    assert_eq!(
        evolved.starts()[..6],
        [
            "trial of 0",
            "trial of 0",
            "trial of 0",
            "trial of 0",
            "diversification of 1 by 0",
            "trial of 1",
        ]
    );
}

#[test]
fn a_directed_evolution_starts_exactly_as_many_diversifications_as_the_most_it_was_given() {
    let evolved = given::started(Double::default().changing(), with_three_diversifications);
    assert_eq!(started(&evolved, "diversification"), 3);
}

#[test]
fn a_diversification_that_fails_is_one_of_the_most_a_directed_evolution_starts() {
    let docker = Double::default().answering(&DIVERSIFICATION, &[Answer::Exited(1)]);
    let evolved = given::started(docker, with_two_diversifications_over_one_task);
    assert_eq!(started(&evolved, "diversification"), 2);
}

#[test]
fn every_harness_that_is_created_trials_every_task_once_whatever_it_completes() {
    let evolved = given::started(Double::default().changing(), given::unchanged);
    assert_eq!(
        trials_started(&evolved),
        [
            (0, "benchmark/t1".to_owned()),
            (0, "benchmark/t2".to_owned()),
            (1, "benchmark/t1".to_owned()),
            (1, "benchmark/t2".to_owned()),
        ]
    );
}

#[test]
fn a_directed_evolution_over_one_task_ends_once_every_created_harness_has_been_given_it() {
    let evolved = given::started(Double::default().changing(), given::over_one_task);
    assert_eq!(started(&evolved, "trial"), 2);
}

#[test]
fn a_directed_evolution_ends_its_journal_with_its_finished_record() {
    assert_eq!(
        said_over_one_task().journal().lines().last(),
        Some(r#"{"record":"directed_evolution_finished","best_harness":1}"#)
    );
}

#[test]
fn a_directed_evolution_says_first_that_it_makes_its_outputs_directory_with_a_copy_of_the_primogenitor()
 {
    let evolved = said_over_one_task();
    assert_eq!(
        evolved.lines()[0],
        format!(
            "cladenc: making the outputs directory {} with a copy of the primogenitor {}",
            evolved.outputs().display(),
            evolved.files().path().join("primogenitor").display()
        )
    );
}

#[test]
fn a_directed_evolution_says_next_that_it_copies_the_tasks_of_its_benchmark() {
    let evolved = said_over_one_task();
    assert_eq!(
        evolved.lines()[1],
        format!(
            "cladenc: copying the 1 task of the benchmark {} into the outputs directory",
            evolved.files().path().join("benchmark").display()
        )
    );
}

#[test]
fn a_directed_evolution_says_a_trial_it_starts_with_no_number() {
    assert_eq!(
        said_over_one_task().lines()[5],
        "cladenc: started the trial of harness 0 on task benchmark/t1"
    );
}

#[test]
fn a_directed_evolution_says_each_action_as_it_starts() {
    assert_eq!(
        said_over_one_task().lines()[7],
        "cladenc: started the diversification of harness 1 by harness 0, diversification 1 of 1"
    );
}

#[test]
fn a_directed_evolution_says_as_many_starts_as_its_journal_records() {
    let evolved = said_over_one_task();
    assert_eq!(
        evolved
            .lines()
            .iter()
            .filter(|line| line.starts_with("cladenc: started the "))
            .count(),
        evolved.journal().matches("_started\"").count()
    );
}

#[test]
fn a_directed_evolution_says_how_a_diversification_ended_and_where_its_child_is_kept() {
    let evolved = said_over_one_task();
    assert!(evolved.lines().contains(&format!(
        "cladenc: the diversification of harness 1 by harness 0 created harness 1, kept in {}",
        evolved.outputs().join("harnesses/1").display()
    )));
}

#[test]
fn a_directed_evolution_says_whether_a_harness_completed_its_task_and_where_what_the_trial_left_is_kept()
 {
    let evolved = said_over_one_task();
    assert!(evolved.lines().contains(&format!(
        "cladenc: harness 1 completed task benchmark/t1: what the trial left is kept in {}",
        evolved.outputs().join("holdout/1/benchmark/t1").display()
    )));
}

#[test]
fn a_directed_evolution_says_last_that_it_finished_and_which_harness_is_its_best() {
    let evolved = said_over_one_task();
    assert_eq!(
        evolved.lines().last(),
        Some(&format!(
            "cladenc: the directed evolution finished with 2 trials started and 2 harnesses: \
             its best harness is harness 1, kept in {}",
            evolved.outputs().join("harnesses/1").display()
        ))
    );
}

#[test]
fn a_directed_evolution_copies_no_record_of_its_journal_to_what_it_says() {
    let said = said_over_one_task().said;
    assert!(!said.contains("\"record\""));
}

#[test]
fn a_directed_evolution_whose_run_record_cannot_be_written_stops_before_any_action() {
    let evolved = given::started(
        Double::default(),
        with_a_primogenitor_whose_path_cannot_be_written,
    );
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::Journal(_))
    ));
}
