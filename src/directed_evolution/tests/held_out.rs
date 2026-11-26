use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{Record, Settings};
use crate::sandboxes::tests::double::{Answer, Double};

const A_TRIAL_ON_A_HOLDOUT_TASK: [&str; 2] = ["run a harness", "trial-0-benchmark+t4"];

fn holding_out(directory: &Memory, settings: &mut Settings) {
    given::over_five_tasks(directory, settings);
    settings.holdout_tasks = vec!["benchmark/t2".to_owned(), "benchmark/t4".to_owned()];
    settings.corroboration_tasks = vec!["benchmark/t4".to_owned()];
}

fn evolved() -> Evolved {
    given::started(Double::default().changing(), holding_out)
}

fn stopped_at_a_holdout_task_and_resumed() -> Evolved {
    let docker = Double::default()
        .changing()
        .answering(&A_TRIAL_ON_A_HOLDOUT_TASK, &[Answer::Exited(125)]);
    let stopped = given::started(docker, holding_out);
    given::resumed(stopped, Double::default().changing())
}

fn tasks_kept_under(evolved: &Evolved, kept: &str) -> [Vec<String>; 2] {
    ["0", "1"].map(|harness| {
        let benchmark = evolved.outputs().join(kept).join(harness).join("benchmark");
        evolved.files().names_in(benchmark)
    })
}

fn best_harness_where_the_child_completes(task: &'static str) -> Option<u64> {
    let docker = Double::default()
        .changing()
        .verifying(move |trial| trial == format!("trial-1-benchmark+{task}"));
    given::started(docker, holding_out).best_harness()
}

#[test]
fn every_harness_trials_the_holdout_tasks_as_well_as_the_others() {
    let finished = evolved()
        .records()
        .into_iter()
        .filter(|record| matches!(record, Record::TrialFinished { .. }))
        .count();
    assert_eq!(finished, 10);
}

#[test]
fn the_trials_of_each_harness_at_the_holdout_tasks_are_kept_under_holdout() {
    assert_eq!(
        tasks_kept_under(&evolved(), "holdout"),
        [vec!["t2", "t4"], vec!["t2", "t4"]]
    );
}

#[test]
fn the_trials_of_each_harness_at_the_non_holdout_tasks_are_kept_under_trials() {
    assert_eq!(
        tasks_kept_under(&evolved(), "trials"),
        [vec!["t1", "t3", "t5"], vec!["t1", "t3", "t5"]]
    );
}

#[test]
fn what_is_said_of_a_trial_on_a_holdout_task_names_where_the_trial_is_kept() {
    let evolved = evolved();
    assert!(evolved.lines().contains(&format!(
        "cladenc: harness 0 did not complete task benchmark/t2, as the verifier wrote a \
         verdict that is not 1: what the trial left is kept in {}",
        evolved.outputs().join("holdout/0/benchmark/t2").display()
    )));
}

#[test]
fn a_resumed_directed_evolution_keeps_the_trial_it_carries_out_again_under_holdout() {
    assert_eq!(
        tasks_kept_under(&stopped_at_a_holdout_task_and_resumed(), "holdout"),
        [vec!["t2", "t4"], vec!["t2", "t4"]]
    );
}

#[test]
fn a_resumed_directed_evolution_keeps_no_holdout_task_where_a_progenitor_reads() {
    assert_eq!(
        tasks_kept_under(&stopped_at_a_holdout_task_and_resumed(), "trials"),
        [vec!["t1", "t3", "t5"], vec!["t1", "t3", "t5"]]
    );
}

#[test]
fn a_child_that_completes_a_selection_task_its_progenitor_did_not_is_named_the_best_harness() {
    assert_eq!(best_harness_where_the_child_completes("t2"), Some(1));
}

#[test]
fn a_child_that_completes_only_a_corroboration_task_is_not_named_the_best_harness() {
    assert_eq!(best_harness_where_the_child_completes("t4"), Some(0));
}

#[test]
fn a_child_that_completes_only_one_more_task_that_is_not_held_out_is_not_named_the_best_harness() {
    assert_eq!(best_harness_where_the_child_completes("t1"), Some(0));
}
