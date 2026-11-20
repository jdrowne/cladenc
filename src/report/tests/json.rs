use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy};
use crate::journal::{Seconds, Settings};
use crate::report::Report;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        genealogy_growth: 0.6,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        attempt_timeout: 60,
        holdout_percentage: 50.0,
        holdout_tasks: vec!["benchmark/t1".to_owned(), "benchmark/t3".to_owned()],
        holdout_corroboration_percentage: 50.0,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        seed: 7,
    }
}

#[test]
fn the_object_holds_the_values_of_the_run_and_of_each_harness() {
    let mut genealogy = settings().genealogy();
    for (task, result) in [
        ("benchmark/t1", AttemptResult::Completed),
        ("benchmark/t2", AttemptResult::NotCompleted),
        ("benchmark/t3", AttemptResult::Completed),
    ] {
        genealogy.start_attempt(0, task).unwrap();
        genealogy.end_attempt(0, task, result).unwrap();
    }
    genealogy.start_procreation(0).unwrap();
    let seconds = [Seconds {
        attempts: 4.5,
        procreation: None,
    }];
    assert_eq!(
        Report::of(&settings(), &genealogy, &seconds).json(),
        concat!(
            r#"{"model":"a-model","model_url":"http://models.example/v1","#,
            r#""procreations_started":1,"max_procreations":800,"best_harness":0,"#,
            r#""selection_tasks_completed_by_any":1,"selection_tasks":1,"#,
            r#""corroboration_tasks_completed_by_any":1,"corroboration_tasks":1,"#,
            r#""other_tasks_completed_by_any":0,"other_tasks":1,"#,
            r#""corroboration_lead":null,"#,
            r#""experiments":{"conducted":0,"progenitors_child_created":0,"#,
            r#""singleton_control_group_harness_created":0,"both_created":0,"#,
            r#""only_progenitors_child_completed":0,"#,
            r#""only_singleton_control_group_harness_completed":0,"likelihood_by_luck":1.0},"#,
            r#""harnesses":["#,
            r#"{"id":0,"progenitor":null,"experiment":null,"state":"created","#,
            r#""selection_completed":1,"selection_not_completed":0,"#,
            r#""corroboration_completed":1,"corroboration_not_completed":0,"#,
            r#""other_completed":0,"other_not_completed":1,"#,
            r#""clade_completed":1,"clade_not_completed":0,"clade_score":1.0,"#,
            r#""attempt_seconds":4.5,"procreation_seconds":null},"#,
            r#"{"id":1,"progenitor":0,"experiment":null,"state":"being_created","#,
            r#""selection_completed":0,"selection_not_completed":0,"#,
            r#""corroboration_completed":0,"corroboration_not_completed":0,"#,
            r#""other_completed":0,"other_not_completed":0,"#,
            r#""clade_completed":0,"clade_not_completed":0,"clade_score":null,"#,
            r#""attempt_seconds":0.0,"procreation_seconds":null}]}"#
        )
    );
}

#[test]
fn the_state_of_a_failed_harness_is_written_as_failed() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Failed)
        .unwrap();
    assert!(
        Report::of(&settings(), &genealogy, &[])
            .json()
            .contains(r#""id":1,"progenitor":0,"experiment":null,"state":"failed""#)
    );
}

#[test]
fn the_object_gives_no_lower_bound() {
    assert!(
        !Report::of(&settings(), &Genealogy::new(), &[])
            .json()
            .contains("lower_bound")
    );
}
