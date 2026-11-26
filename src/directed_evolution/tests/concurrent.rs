use crate::directed_evolution::tests::given;
use crate::file_system::tests::double::Memory;
use crate::journal::{Record, Settings};
use crate::sandboxes::tests::double::Double;

fn three_at_once(directory: &Memory, settings: &mut Settings) {
    given::over_five_tasks(directory, settings);
    settings.max_concurrent_sandboxes = 3;
}

fn three_at_once_with_two_diversifications(directory: &Memory, settings: &mut Settings) {
    three_at_once(directory, settings);
    settings.max_diversifications = 2;
}

fn two_at_once(directory: &Memory, settings: &mut Settings) {
    given::over_five_tasks(directory, settings);
    settings.max_concurrent_sandboxes = 2;
}

fn records(change: impl FnOnce(&Memory, &mut Settings)) -> Vec<Record> {
    given::started(Double::default().changing(), change).records()
}

fn most_running_at_once(records: &[Record]) -> usize {
    let mut running = 0_usize;
    let mut most = 0;
    for record in records {
        match record {
            Record::DiversificationStarted { .. }
            | Record::BaselineStarted { .. }
            | Record::TrialStarted { .. } => {
                running += 1;
                most = most.max(running);
            }
            Record::DiversificationFinished { .. }
            | Record::BaselineFinished { .. }
            | Record::TrialFinished { .. } => running -= 1,
            Record::Run { .. }
            | Record::PrimogenitorBuilt
            | Record::DirectedEvolutionFinished { .. } => {}
        }
    }
    most
}

fn most_running_as_a_diversification_starts(records: &[Record]) -> Option<usize> {
    let mut running = 0_usize;
    let mut most = None;
    for record in records {
        match record {
            Record::DiversificationStarted { .. } => {
                most = most.max(Some(running));
                running += 1;
            }
            Record::BaselineStarted { .. } | Record::TrialStarted { .. } => running += 1,
            Record::DiversificationFinished { .. }
            | Record::BaselineFinished { .. }
            | Record::TrialFinished { .. } => running -= 1,
            Record::Run { .. }
            | Record::PrimogenitorBuilt
            | Record::DirectedEvolutionFinished { .. } => {}
        }
    }
    most
}

fn count(records: &[Record], counted: impl Fn(&Record) -> bool) -> usize {
    records.iter().filter(|record| counted(record)).count()
}

fn fewest_tasks_a_progenitor_had_trialled(records: &[Record]) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .filter_map(|(place, record)| match record {
            Record::DiversificationStarted { progenitor, .. } => {
                Some(count(&records[..place], |earlier| {
                    matches!(
                        earlier,
                        Record::TrialFinished { harness, .. } if harness == progenitor
                    )
                }))
            }
            _ => None,
        })
        .min()
}

#[test]
fn with_a_limit_of_three_the_journal_holds_three_actions_started_and_not_finished_at_most() {
    assert_eq!(most_running_at_once(&records(three_at_once)), 3);
}

#[test]
fn with_a_limit_of_three_the_journal_still_holds_one_trial_of_every_task_by_each_harness() {
    let records = records(three_at_once);
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::TrialStarted { .. }
        )),
        10
    );
}

#[test]
fn with_a_limit_of_three_every_diversification_still_starts() {
    let records = records(three_at_once_with_two_diversifications);
    assert_eq!(
        count(&records, |record| matches!(
            record,
            Record::DiversificationStarted { .. }
        )),
        2
    );
}

#[test]
fn with_a_limit_of_three_a_diversification_starts_only_when_nothing_else_runs() {
    let records = records(three_at_once_with_two_diversifications);
    assert_eq!(most_running_as_a_diversification_starts(&records), Some(0));
}

#[test]
fn with_a_limit_of_two_no_diversification_starts_before_its_progenitor_is_benchmarked_in_full() {
    assert_eq!(
        fewest_tasks_a_progenitor_had_trialled(&records(two_at_once)),
        Some(5)
    );
}
