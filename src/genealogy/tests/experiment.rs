use crate::genealogy::{
    CreationOutcome, Experiment, Genealogy, GenealogyError, HarnessId, HarnessState, TrialResult,
};

const EXPERIMENT: Experiment = Experiment {
    child: 2,
    progenitor: 1,
};

fn child_of(genealogy: &mut Genealogy, progenitor: HarnessId) -> HarnessId {
    let child = genealogy.start_diversification(progenitor).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    child
}

fn with_a_child_of_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new().selecting_by(&["a".to_owned()]);
    child_of(&mut genealogy, 0);
    child_of(&mut genealogy, 1);
    genealogy
}

fn with_a_baseline_started() -> Genealogy {
    let mut genealogy = with_a_child_of_a_child();
    genealogy.start_baseline(EXPERIMENT).unwrap();
    genealogy
}

fn with_a_baseline_ended(outcome: CreationOutcome) -> Genealogy {
    let mut genealogy = with_a_baseline_started();
    genealogy.end_baseline(3, outcome).unwrap();
    genealogy
}

fn after_a_trial_of_the_singleton_control_group_harness(result: TrialResult) -> Genealogy {
    let mut genealogy = with_a_baseline_ended(CreationOutcome::Created);
    genealogy.start_trial(3, "a").unwrap();
    genealogy.end_trial(3, "a", result).unwrap();
    genealogy
}

#[test]
fn no_experiment_awaits_its_baseline_in_a_new_genealogy() {
    assert_eq!(Genealogy::new().experiment_awaiting_its_baseline(), None);
}

#[test]
fn no_experiment_awaits_its_baseline_where_the_primogenitor_is_cloned() {
    let mut genealogy = Genealogy::new();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(genealogy.experiment_awaiting_its_baseline(), None);
}

#[test]
fn an_experiment_awaits_its_baseline_where_a_harness_other_than_the_primogenitor_is_cloned() {
    assert_eq!(
        with_a_child_of_a_child().experiment_awaiting_its_baseline(),
        Some(EXPERIMENT)
    );
}

#[test]
fn an_experiment_whose_baseline_has_started_awaits_it_no_longer() {
    assert_eq!(
        with_a_baseline_started().experiment_awaiting_its_baseline(),
        None
    );
}

#[test]
fn the_singleton_control_group_harness_takes_the_next_id() {
    assert_eq!(with_a_child_of_a_child().start_baseline(EXPERIMENT), Ok(3));
}

#[test]
fn the_singleton_control_group_harness_is_nobody_s_child() {
    assert_eq!(with_a_baseline_started().harnesses()[3].progenitor(), None);
}

#[test]
fn the_singleton_control_group_harness_carries_its_experiment() {
    assert_eq!(
        with_a_baseline_started().harnesses()[3].experiment(),
        Some(EXPERIMENT)
    );
}

#[test]
fn a_child_carries_no_experiment() {
    assert_eq!(with_a_baseline_started().harnesses()[2].experiment(), None);
}

#[test]
fn the_singleton_control_group_harness_is_being_created_while_its_baseline_is_established() {
    assert_eq!(
        with_a_baseline_started().harnesses()[3].state(),
        HarnessState::BeingCreated
    );
}

#[test]
fn a_baseline_that_created_the_singleton_control_group_harness_leaves_a_created_harness() {
    assert_eq!(
        with_a_baseline_ended(CreationOutcome::Created).harnesses()[3].state(),
        HarnessState::Created
    );
}

#[test]
fn a_baseline_that_failed_leaves_a_failed_harness() {
    assert_eq!(
        with_a_baseline_ended(CreationOutcome::Failed).harnesses()[3].state(),
        HarnessState::Failed
    );
}

#[test]
fn a_baseline_that_left_the_source_code_of_the_progenitor_unchanged_leaves_an_unchanged_harness() {
    assert_eq!(
        with_a_baseline_ended(CreationOutcome::Unchanged).harnesses()[3].state(),
        HarnessState::Unchanged
    );
}

#[test]
fn a_baseline_is_refused_where_the_primogenitor_was_cloned() {
    let mut genealogy = Genealogy::new();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(
        genealogy.start_baseline(Experiment {
            child: 1,
            progenitor: 0
        }),
        Err(GenealogyError::ExperimentNotAwaitingBaseline {
            child: 1,
            progenitor: 0
        })
    );
}

#[test]
fn a_second_baseline_of_one_experiment_is_refused() {
    assert_eq!(
        with_a_baseline_started().start_baseline(EXPERIMENT),
        Err(GenealogyError::ExperimentNotAwaitingBaseline {
            child: 2,
            progenitor: 1
        })
    );
}

#[test]
fn a_baseline_that_names_another_progenitor_than_the_one_that_was_cloned_is_refused() {
    assert_eq!(
        with_a_child_of_a_child().start_baseline(Experiment {
            child: 2,
            progenitor: 0
        }),
        Err(GenealogyError::ExperimentNotAwaitingBaseline {
            child: 2,
            progenitor: 0
        })
    );
}

#[test]
fn the_end_of_a_baseline_is_refused_for_a_child_that_is_being_created() {
    let mut genealogy = Genealogy::new();
    genealogy.start_diversification(0).unwrap();
    assert_eq!(
        genealogy.end_baseline(1, CreationOutcome::Created),
        Err(GenealogyError::SingletonControlGroupHarnessNotBeingCreated { harness: 1 })
    );
}

#[test]
fn the_end_of_a_baseline_is_refused_once_the_baseline_has_ended() {
    assert_eq!(
        with_a_baseline_ended(CreationOutcome::Created).end_baseline(3, CreationOutcome::Created),
        Err(GenealogyError::SingletonControlGroupHarnessNotBeingCreated { harness: 3 })
    );
}

#[test]
fn the_end_of_a_diversification_is_refused_for_a_singleton_control_group_harness() {
    assert_eq!(
        with_a_baseline_started().end_diversification(3, CreationOutcome::Created),
        Err(GenealogyError::ChildNotBeingCreated { child: 3 })
    );
}

#[test]
fn a_baseline_is_not_counted_as_a_diversification() {
    assert_eq!(with_a_baseline_started().diversifications_started(), 2);
}

#[test]
fn harness_count_counts_the_singleton_control_group_harness() {
    assert_eq!(with_a_baseline_started().harness_count(), 4);
}

#[test]
fn a_child_made_after_a_baseline_takes_the_id_after_the_singleton_control_group_harness() {
    assert_eq!(with_a_baseline_started().start_diversification(0), Ok(4));
}

#[test]
fn a_selection_task_the_singleton_control_group_harness_completed_is_counted_for_it() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[3].selection_passed(), 1);
}

#[test]
fn the_singleton_control_group_harness_is_counted_in_no_clade_of_its_own() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[3].clade_completion_rate(), None);
}

#[test]
fn the_singleton_control_group_harness_is_not_counted_in_the_clade_of_the_progenitor() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Fail);
    assert_eq!(genealogy.harnesses()[1].clade_completion_rate(), None);
}

#[test]
fn the_singleton_control_group_harness_is_not_counted_in_the_clade_of_the_primogenitor() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert_eq!(genealogy.harnesses()[0].clade_completion_rate(), None);
}

#[test]
fn a_trial_of_the_singleton_control_group_harness_counts_as_started() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert_eq!(genealogy.started(), 1);
}

#[test]
fn a_trial_of_the_singleton_control_group_harness_is_no_action_that_is_counted_as_started() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert_eq!(genealogy.actions_started(), 2);
}

#[test]
fn the_actions_started_count_each_diversification_and_each_trial_of_a_harness_that_is_not_one() {
    let mut genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    genealogy.start_trial(0, "a").unwrap();
    assert_eq!(genealogy.actions_started(), 3);
}

#[test]
fn a_task_only_the_singleton_control_group_harness_completed_is_completed_by_no_harness_of_a_clade()
{
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Pass);
    assert!(!genealogy.completed_by_any("a"));
}

#[test]
fn a_harness_has_trialled_a_task_once_its_trial_has_ended() {
    let genealogy = after_a_trial_of_the_singleton_control_group_harness(TrialResult::Fail);
    assert!(genealogy.harnesses()[3].has_trialled("a"));
}

#[test]
fn a_harness_has_not_trialled_a_task_while_its_trial_is_running() {
    let mut genealogy = with_a_baseline_ended(CreationOutcome::Created);
    genealogy.start_trial(3, "a").unwrap();
    assert!(!genealogy.harnesses()[3].has_trialled("a"));
}
