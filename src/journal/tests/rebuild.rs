use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy};
use crate::journal::{self, Journal, Record, Seconds, Settings, Started, State};
use tempfile::TempDir;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn dividing_its_tasks() -> Settings {
    Settings {
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
        }],
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        ..settings()
    }
}

fn an_attempt_at_each_task() -> [Record; 6] {
    [
        attempt_started(0, "benchmark/t1"),
        attempt_finished(0, "benchmark/t1", AttemptResult::NotCompleted),
        attempt_started(0, "benchmark/t2"),
        attempt_finished(0, "benchmark/t2", AttemptResult::Completed),
        attempt_started(0, "benchmark/t3"),
        attempt_finished(0, "benchmark/t3", AttemptResult::TimedOut),
    ]
}

fn procreation_started(child: u64, progenitor: u64) -> Record {
    Record::ProcreationStarted { child, progenitor }
}

fn procreation_finished(child: u64, outcome: CreationOutcome) -> Record {
    Record::ProcreationFinished {
        child,
        outcome,
        seconds: 30.25,
    }
}

fn attempt_started(harness: u64, task: &str) -> Record {
    Record::AttemptStarted {
        harness,
        task: task.to_owned(),
    }
}

fn attempt_finished(harness: u64, task: &str, result: AttemptResult) -> Record {
    Record::AttemptFinished {
        harness,
        task: task.to_owned(),
        result,
        seconds: 1.5,
    }
}

fn journal_of(records: &[Record]) -> TempDir {
    journal_with(settings(), records)
}

fn journal_with(settings: Settings, records: &[Record]) -> TempDir {
    let outputs_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(outputs_directory.path()).unwrap();
    journal
        .append(&Record::Run {
            format: journal::FORMAT,
            settings: Box::new(settings),
        })
        .unwrap();
    for record in records {
        journal.append(record).unwrap();
    }
    outputs_directory
}

fn rebuilt(records: &[Record]) -> State {
    journal::read(journal_of(records).path()).unwrap()
}

fn rebuilt_of_a_directed_evolution_that_builds(records: &[Record]) -> State {
    let settings = Settings {
        harness_build_command: Some("make harness".to_owned()),
        ..settings()
    };
    journal::read(journal_with(settings, records).path()).unwrap()
}

fn rebuilt_with_a_last_line_of(records: &[Record], last_line: &str) -> State {
    let outputs_directory = journal_of(records);
    OpenOptions::new()
        .append(true)
        .open(journal::path(outputs_directory.path()))
        .unwrap()
        .write_all(last_line.as_bytes())
        .unwrap();
    journal::read(outputs_directory.path()).unwrap()
}

fn four_actions_of_which_two_finished() -> [Record; 6] {
    [
        procreation_started(1, 0),
        attempt_started(0, "t1"),
        procreation_started(2, 0),
        attempt_started(0, "t2"),
        procreation_finished(1, CreationOutcome::Created),
        attempt_finished(0, "t1", AttemptResult::Completed),
    ]
}

#[test]
fn the_settings_of_the_run_record_come_back() {
    assert_eq!(rebuilt(&[]).settings, settings());
}

#[test]
fn a_journal_that_holds_no_action_gives_a_genealogy_of_the_primogenitor_alone() {
    assert_eq!(rebuilt(&[]).genealogy, Genealogy::new());
}

#[test]
fn a_journal_that_names_a_build_command_and_holds_no_build_gives_a_primogenitor_awaiting_its_build()
{
    assert_eq!(
        rebuilt_of_a_directed_evolution_that_builds(&[]).genealogy,
        Genealogy::unbuilt()
    );
}

#[test]
fn a_journal_that_holds_the_build_of_its_primogenitor_gives_a_created_primogenitor() {
    assert_eq!(
        rebuilt_of_a_directed_evolution_that_builds(&[Record::PrimogenitorBuilt]).genealogy,
        Genealogy::new()
    );
}

#[test]
fn the_build_of_the_primogenitor_is_not_counted_as_an_action() {
    assert_eq!(
        rebuilt_of_a_directed_evolution_that_builds(&[Record::PrimogenitorBuilt]).next_action,
        0
    );
}

#[test]
fn rebuilding_gives_the_genealogy_that_the_same_changes_give_directly() {
    let mut genealogy = Genealogy::new();
    genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(1, CreationOutcome::Created)
        .unwrap();
    genealogy.start_attempt(1, "t2").unwrap();
    genealogy
        .end_attempt(1, "t2", AttemptResult::Completed)
        .unwrap();
    genealogy.start_procreation(1).unwrap();
    genealogy.start_attempt(0, "t1").unwrap();
    genealogy
        .end_attempt(0, "t1", AttemptResult::TimedOut)
        .unwrap();
    genealogy
        .end_procreation(2, CreationOutcome::Failed)
        .unwrap();
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Created),
        attempt_started(1, "t2"),
        attempt_finished(1, "t2", AttemptResult::Completed),
        procreation_started(2, 1),
        attempt_started(0, "t1"),
        attempt_finished(0, "t1", AttemptResult::TimedOut),
        procreation_finished(2, CreationOutcome::Failed),
    ]);
    assert_eq!(state.genealogy, genealogy);
}

#[test]
fn rebuilding_counts_the_attempts_at_the_selection_tasks_of_the_run_record_apart() {
    let state =
        journal::read(journal_with(dividing_its_tasks(), &an_attempt_at_each_task()).path())
            .unwrap();
    let primogenitor = &state.genealogy.harnesses()[0];
    assert_eq!(
        (
            primogenitor.selection_completed(),
            primogenitor.non_holdout_not_completed()
        ),
        (1, 1)
    );
}

#[test]
fn rebuilding_counts_the_attempts_at_the_corroboration_tasks_of_the_run_record_apart() {
    let state =
        journal::read(journal_with(dividing_its_tasks(), &an_attempt_at_each_task()).path())
            .unwrap();
    let primogenitor = &state.genealogy.harnesses()[0];
    assert_eq!(
        (
            primogenitor.corroboration_completed(),
            primogenitor.corroboration_not_completed()
        ),
        (0, 1)
    );
}

#[test]
fn rebuilding_counts_no_attempt_at_a_corroboration_task_in_a_clade() {
    let state =
        journal::read(journal_with(dividing_its_tasks(), &an_attempt_at_each_task()).path())
            .unwrap();
    let primogenitor = &state.genealogy.harnesses()[0];
    assert_eq!(
        (
            primogenitor.clade_completed(),
            primogenitor.clade_not_completed()
        ),
        (1, 0)
    );
}

#[test]
fn a_journal_that_holds_no_action_gives_the_primogenitor_no_seconds_spent() {
    assert_eq!(rebuilt(&[]).seconds, [Seconds::default()]);
}

#[test]
fn rebuilding_adds_up_the_seconds_of_the_attempts_of_each_harness() {
    let state = rebuilt(&[
        attempt_started(0, "t1"),
        attempt_finished(0, "t1", AttemptResult::Completed),
        attempt_started(0, "t2"),
        attempt_finished(0, "t2", AttemptResult::NotCompleted),
    ]);
    assert_eq!(state.seconds[0].attempts, 3.0);
}

#[test]
fn rebuilding_gives_a_child_the_seconds_of_the_procreation_that_made_it() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Created),
    ]);
    assert_eq!(state.seconds[1].procreation, Some(30.25));
}

#[test]
fn rebuilding_gives_a_child_whose_procreation_has_not_ended_no_seconds_of_it() {
    let state = rebuilt(&[procreation_started(1, 0)]);
    assert_eq!(state.seconds[1], Seconds::default());
}

#[test]
fn rebuilding_gives_the_seconds_of_a_procreation_to_the_child_and_not_to_its_progenitor() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Failed),
    ]);
    assert_eq!(state.seconds[0].procreation, None);
}

#[test]
fn the_actions_that_did_not_finish_come_back_in_the_order_they_started() {
    assert_eq!(
        rebuilt(&four_actions_of_which_two_finished()).unfinished,
        [
            Started::Procreation {
                child: 2,
                progenitor: 0
            },
            Started::Attempt {
                harness: 0,
                task: "t2".to_owned()
            },
        ]
    );
}

#[test]
fn a_journal_whose_actions_all_finished_holds_no_unfinished_action() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Created),
        attempt_started(1, "t1"),
        attempt_finished(1, "t1", AttemptResult::NotCompleted),
    ]);
    assert_eq!(state.unfinished, []);
}

#[test]
fn the_number_of_the_next_action_is_the_count_of_started_records() {
    assert_eq!(
        rebuilt(&four_actions_of_which_two_finished()).next_action,
        4
    );
}

#[test]
fn the_first_action_of_a_journal_that_holds_no_action_takes_number_0() {
    assert_eq!(rebuilt(&[]).next_action, 0);
}

#[test]
fn a_journal_that_holds_the_end_of_its_directed_evolution_names_the_best_harness() {
    let state = rebuilt(&[
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Created),
        Record::DirectedEvolutionFinished { best_harness: 1 },
    ]);
    assert_eq!(state.best_harness, Some(1));
}

#[test]
fn a_journal_that_does_not_hold_the_end_of_its_directed_evolution_names_no_best_harness() {
    assert_eq!(rebuilt(&[procreation_started(1, 0)]).best_harness, None);
}

#[test]
fn a_cut_short_last_line_is_ignored_when_the_journal_is_read() {
    let state = rebuilt_with_a_last_line_of(
        &[procreation_started(1, 0)],
        "{\"record\":\"procreation_finished\",\"chi",
    );
    assert_eq!(state, rebuilt(&[procreation_started(1, 0)]));
}

#[test]
fn a_whole_record_with_no_newline_at_its_end_is_ignored_as_cut_short() {
    let state = rebuilt_with_a_last_line_of(
        &[],
        "{\"record\":\"directed_evolution_finished\",\"best_harness\":0}",
    );
    assert_eq!(state.best_harness, None);
}

#[test]
fn reading_a_journal_takes_no_lock() {
    let outputs_directory = journal_of(&[]);
    let _writer = Journal::open(outputs_directory.path()).unwrap();
    assert!(journal::read(outputs_directory.path()).is_ok());
}

fn an_experiment_whose_baseline_started() -> Vec<Record> {
    vec![
        procreation_started(1, 0),
        procreation_finished(1, CreationOutcome::Created),
        procreation_started(2, 1),
        procreation_finished(2, CreationOutcome::Created),
        Record::BaselineStarted {
            harness: 3,
            child: 2,
            progenitor: 1,
        },
    ]
}

fn an_experiment_whose_baseline_ended_and_then(records: &[Record]) -> Vec<Record> {
    let mut experiment = an_experiment_whose_baseline_started();
    experiment.push(Record::BaselineFinished {
        harness: 3,
        outcome: CreationOutcome::Created,
        seconds: 12.5,
    });
    experiment.extend_from_slice(records);
    experiment
}

#[test]
fn a_baseline_that_has_not_ended_comes_back_as_an_action_that_did_not_finish() {
    assert_eq!(
        rebuilt(&an_experiment_whose_baseline_started()).unfinished,
        [Started::Baseline {
            harness: 3,
            child: 2,
            progenitor: 1
        }]
    );
}

#[test]
fn a_baseline_that_ended_is_no_action_that_did_not_finish() {
    assert_eq!(
        rebuilt(&an_experiment_whose_baseline_ended_and_then(&[])).unfinished,
        []
    );
}

#[test]
fn the_start_of_a_baseline_takes_no_number_of_an_action() {
    assert_eq!(
        rebuilt(&an_experiment_whose_baseline_started()).next_action,
        2
    );
}

#[test]
fn an_attempt_of_a_singleton_control_group_harness_takes_no_number_of_an_action() {
    let records =
        an_experiment_whose_baseline_ended_and_then(&[attempt_started(3, "benchmark/t1")]);
    assert_eq!(rebuilt(&records).next_action, 2);
}

#[test]
fn rebuilding_gives_a_singleton_control_group_harness_no_seconds_of_a_procreation() {
    let state = rebuilt(&an_experiment_whose_baseline_ended_and_then(&[]));
    assert_eq!(state.seconds[3].procreation, None);
}

#[test]
fn rebuilding_gives_a_singleton_control_group_harness_the_seconds_of_its_attempts() {
    let state = rebuilt(&an_experiment_whose_baseline_ended_and_then(&[
        attempt_started(3, "benchmark/t1"),
        attempt_finished(3, "benchmark/t1", AttemptResult::Completed),
    ]));
    assert_eq!(state.seconds[3].attempts, 1.5);
}

#[test]
fn the_end_of_a_baseline_leaves_the_other_actions_that_did_not_finish() {
    let mut records = an_experiment_whose_baseline_started();
    records.push(attempt_started(2, "benchmark/t1"));
    records.push(Record::BaselineFinished {
        harness: 3,
        outcome: CreationOutcome::Failed,
        seconds: 12.5,
    });
    assert_eq!(
        rebuilt(&records).unfinished,
        [Started::Attempt {
            harness: 2,
            task: "benchmark/t1".to_owned()
        }]
    );
}
