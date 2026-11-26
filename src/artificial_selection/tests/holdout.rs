use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{CreationOutcome, Genealogy, HarnessId, TrialResult};

fn tasks() -> Vec<String> {
    (0..15).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_diversifications: 1000,
        tasks: tasks(),
        seed,
    }
}

fn benchmarked(genealogy: &mut Genealogy, harness: HarnessId, completed: &[usize]) {
    for (number, task) in tasks().iter().enumerate() {
        let result = if completed.contains(&number) {
            TrialResult::Completed
        } else {
            TrialResult::NotCompleted
        };
        genealogy.start_trial(harness, task).unwrap();
        genealogy.end_trial(harness, task, result).unwrap();
    }
}

fn two_children_selected_by_the_first_five_tasks(first: &[usize], second: &[usize]) -> Genealogy {
    let mut genealogy = Genealogy::new()
        .selecting_by(&tasks()[..5])
        .corroborating_with(&tasks()[10..]);
    benchmarked(&mut genealogy, 0, &[]);
    for completed in [first, second] {
        let child = genealogy.start_diversification(0).unwrap();
        genealogy
            .end_diversification(child, CreationOutcome::Created)
            .unwrap();
        benchmarked(&mut genealogy, child, completed);
    }
    genealogy
}

fn best(genealogy: &Genealogy) -> HarnessId {
    artificial_selection(0).best_harness(genealogy)
}

fn times_progenitor(genealogy: &Genealogy, progenitor: HarnessId) -> usize {
    (0..200)
        .filter(|seed| {
            artificial_selection(*seed).next_action(genealogy, 0)
                == Some(Action::Diversification { progenitor })
        })
        .count()
}

#[test]
fn the_best_harness_completed_the_most_selection_tasks_and_not_the_most_tasks() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[5, 6, 7, 8, 9], &[0]);
    assert_eq!(best(&genealogy), 2);
}

#[test]
fn tasks_that_are_not_held_out_break_no_tie_between_harnesses_with_as_many_selection_tasks() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[0], &[0, 5, 6, 7, 8, 9]);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_harness_that_completed_only_tasks_that_are_not_held_out_is_not_the_best_for_them() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[5, 6, 7, 8, 9], &[]);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn a_harness_that_completed_more_selection_tasks_is_the_progenitor_more_often() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[0, 1, 2, 3, 4], &[]);
    assert!(times_progenitor(&genealogy, 1) > times_progenitor(&genealogy, 2));
}

#[test]
fn tasks_that_are_not_held_out_do_not_make_a_harness_the_progenitor_more_often() {
    let genealogy =
        two_children_selected_by_the_first_five_tasks(&[5, 6, 7, 8, 9], &[0, 1, 2, 3, 4]);
    assert!(times_progenitor(&genealogy, 2) > times_progenitor(&genealogy, 1));
}

#[test]
fn the_progenitor_is_drawn_the_same_whatever_was_completed_of_the_tasks_that_are_not_held_out() {
    let plain = two_children_selected_by_the_first_five_tasks(&[0], &[1, 2]);
    let with_others = two_children_selected_by_the_first_five_tasks(&[0, 5, 6, 7], &[1, 2, 8, 9]);
    assert!((0..50).all(|seed| {
        artificial_selection(seed).next_action(&plain, 0)
            == artificial_selection(seed).next_action(&with_others, 0)
    }));
}

#[test]
fn the_best_harness_completed_the_most_selection_tasks_and_not_the_most_corroboration_tasks() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[10, 11, 12, 13, 14], &[0]);
    assert_eq!(best(&genealogy), 2);
}

#[test]
fn corroboration_tasks_break_no_tie_between_harnesses_with_as_many_selection_tasks() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[0], &[0, 10, 11, 12, 13, 14]);
    assert_eq!(best(&genealogy), 1);
}

#[test]
fn a_harness_that_completed_only_corroboration_tasks_is_not_the_best_for_them() {
    let genealogy = two_children_selected_by_the_first_five_tasks(&[10, 11, 12, 13, 14], &[]);
    assert_eq!(best(&genealogy), 0);
}

#[test]
fn corroboration_tasks_do_not_make_a_harness_the_progenitor_more_often() {
    let genealogy =
        two_children_selected_by_the_first_five_tasks(&[10, 11, 12, 13, 14], &[0, 1, 2, 3, 4]);
    assert!(times_progenitor(&genealogy, 2) > times_progenitor(&genealogy, 1));
}

#[test]
fn the_progenitor_is_drawn_the_same_whatever_was_completed_of_the_corroboration_tasks() {
    let plain = two_children_selected_by_the_first_five_tasks(&[0], &[1, 2]);
    let with_corroboration =
        two_children_selected_by_the_first_five_tasks(&[0, 10, 11], &[1, 2, 12, 13, 14]);
    assert!((0..50).all(|seed| {
        artificial_selection(seed).next_action(&plain, 0)
            == artificial_selection(seed).next_action(&with_corroboration, 0)
    }));
}
