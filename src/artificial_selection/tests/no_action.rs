use crate::artificial_selection::ArtificialSelection;
use crate::genealogy::{Genealogy, TrialResult};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_diversifications: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_diversifications,
        tasks: tasks(),
        seed: 0,
    }
}

fn with_the_primogenitor_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    for task in tasks() {
        genealogy.start_trial(0, &task).unwrap();
        genealogy
            .end_trial(0, &task, TrialResult::Completed)
            .unwrap();
    }
    genealogy
}

#[test]
fn there_is_no_next_action_while_the_only_harness_that_could_be_cloned_still_trials_a_task() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    genealogy.start_trial(0, "task-0").unwrap();
    genealogy.start_trial(0, "task-1").unwrap();
    assert_eq!(artificial_selection(100).next_action(&genealogy, 0), None);
}

#[test]
fn there_is_no_next_action_while_the_last_diversification_is_under_way_and_no_task_is_left_to_give()
{
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(artificial_selection(1).next_action(&genealogy, 0), None);
}

#[test]
fn there_is_no_next_action_while_a_diversification_is_under_way_though_more_may_start() {
    let mut genealogy = with_the_primogenitor_benchmarked_in_full();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(artificial_selection(100).next_action(&genealogy, 0), None);
}
