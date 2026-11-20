use crate::artificial_selection::wait::Waits;
use crate::artificial_selection::{Action, ArtificialSelection};
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, Genealogy, HarnessId};

const CREATED: CreationOutcome = CreationOutcome::Created;
const FAILED: CreationOutcome = CreationOutcome::Failed;

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(max_procreations: u64, growth: f64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations,
        growth,
        tasks: tasks(),
        seed: 7,
    }
}

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId, outcome: CreationOutcome) {
    let child = genealogy.start_procreation(progenitor).unwrap();
    genealogy.end_procreation(child, outcome).unwrap();
}

fn singleton_control_group_harness_of(
    genealogy: &mut Genealogy,
    experiment: Experiment,
    outcome: CreationOutcome,
) {
    let harness = genealogy.start_baseline(experiment).unwrap();
    genealogy.end_baseline(harness, outcome).unwrap();
}

fn attempt(genealogy: &mut Genealogy, harness: HarnessId, task: &str, result: AttemptResult) {
    genealogy.start_attempt(harness, task).unwrap();
    genealogy.end_attempt(harness, task, result).unwrap();
}

fn benchmark_in_full(genealogy: &mut Genealogy, harness: HarnessId, result: AttemptResult) {
    for task in tasks() {
        attempt(genealogy, harness, &task, result);
    }
}

fn after_an_experiment() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CREATED);
    child_of(&mut genealogy, 1, CREATED);
    singleton_control_group_harness_of(
        &mut genealogy,
        Experiment {
            child: 2,
            progenitor: 1,
        },
        CREATED,
    );
    genealogy
}

fn with_every_harness_benchmarked_in_full_and_the_singleton_control_group_harness_ahead()
-> Genealogy {
    let mut genealogy = after_an_experiment();
    for harness in [0, 1, 2] {
        benchmark_in_full(&mut genealogy, harness, AttemptResult::NotCompleted);
    }
    benchmark_in_full(&mut genealogy, 3, AttemptResult::Completed);
    genealogy
}

fn with_a_harness_whose_last_two_procreations_created_no_child() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CREATED);
    for child in [2, 4] {
        child_of(&mut genealogy, 1, FAILED);
        singleton_control_group_harness_of(
            &mut genealogy,
            Experiment {
                child,
                progenitor: 1,
            },
            FAILED,
        );
    }
    child_of(&mut genealogy, 0, CREATED);
    genealogy
}

#[test]
fn a_singleton_control_group_harness_with_a_task_left_is_given_it_as_a_child_is() {
    let mut genealogy = after_an_experiment();
    for harness in [0, 1, 2] {
        benchmark_in_full(&mut genealogy, harness, AttemptResult::NotCompleted);
    }
    assert_eq!(
        artificial_selection(2, 1.0).next_action(&genealogy, 0),
        Some(Action::Attempt {
            harness: 3,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_singleton_control_group_harness_is_not_chosen_to_procreate_though_it_alone_is_benchmarked_in_full()
 {
    let mut genealogy = after_an_experiment();
    for harness in [0, 1, 2] {
        attempt(
            &mut genealogy,
            harness,
            "task-0",
            AttemptResult::NotCompleted,
        );
    }
    benchmark_in_full(&mut genealogy, 3, AttemptResult::Completed);
    assert_eq!(
        artificial_selection(1000, 1.0).next_action(&genealogy, 0),
        Some(Action::Attempt {
            harness: 0,
            task: "task-1".to_owned()
        })
    );
}

#[test]
fn a_singleton_control_group_harness_is_not_named_the_best_harness_though_it_completed_the_most() {
    let genealogy =
        with_every_harness_benchmarked_in_full_and_the_singleton_control_group_harness_ahead();
    assert_eq!(artificial_selection(1000, 1.0).best_harness(&genealogy), 0);
}

#[test]
fn the_attempts_of_a_singleton_control_group_harness_make_no_procreation_due() {
    let mut genealogy = after_an_experiment();
    benchmark_in_full(&mut genealogy, 0, AttemptResult::NotCompleted);
    benchmark_in_full(&mut genealogy, 3, AttemptResult::NotCompleted);
    assert_eq!(
        artificial_selection(1000, 1.0).next_action(&genealogy, 0),
        Some(Action::Attempt {
            harness: 1,
            task: "task-0".to_owned()
        })
    );
}

#[test]
fn a_singleton_control_group_harness_is_not_one_of_the_harnesses_a_procreation_waits_for() {
    let mut genealogy = after_an_experiment();
    benchmark_in_full(&mut genealogy, 0, AttemptResult::NotCompleted);
    attempt(&mut genealogy, 1, "task-0", AttemptResult::NotCompleted);
    assert_eq!(
        artificial_selection(1000, 1.0).next_action(&genealogy, 0),
        Some(Action::Procreation { progenitor: 0 })
    );
}

#[test]
fn a_procreation_is_due_while_a_singleton_control_group_harness_is_still_being_created() {
    let mut genealogy = Genealogy::new().selecting_by(&tasks());
    child_of(&mut genealogy, 0, CREATED);
    child_of(&mut genealogy, 1, CREATED);
    genealogy
        .start_baseline(Experiment {
            child: 2,
            progenitor: 1,
        })
        .unwrap();
    for harness in [0, 1, 2] {
        benchmark_in_full(&mut genealogy, harness, AttemptResult::NotCompleted);
    }
    assert!(matches!(
        artificial_selection(1000, 0.0).next_action(&genealogy, 0),
        Some(Action::Procreation { .. })
    ));
}

#[test]
fn a_harness_is_left_out_by_the_number_of_a_procreation_and_not_by_the_id_of_a_harness() {
    let genealogy = with_a_harness_whose_last_two_procreations_created_no_child();
    assert!(Waits::of(&genealogy).has_one_for(&genealogy.harnesses()[1]));
}
