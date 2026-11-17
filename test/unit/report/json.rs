use std::path::PathBuf;

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::{AttemptResult, Genealogy, ProcreationOutcome};
use cladenc::journal::Settings;
use cladenc::report::Report;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/primogenitor"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned()],
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
        seed: 7,
    }
}

#[test]
fn the_object_holds_the_values_of_the_run_and_of_each_harness() {
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "t1").unwrap();
    genealogy
        .end_attempt(0, "t1", AttemptResult::Completed)
        .unwrap();
    genealogy.start_procreation(0).unwrap();
    assert_eq!(
        Report::of(&settings(), &genealogy).json(),
        concat!(
            r#"{"model":"a-model","model_url":"http://models.example/v1","#,
            r#""procreations_started":1,"max_procreations":800,"best_harness":0,"harnesses":["#,
            r#"{"id":0,"progenitor":null,"state":"created","completed":1,"not_completed":0,"#,
            r#""clade_completed":1,"clade_not_completed":0,"clade_score":1.0},"#,
            r#"{"id":1,"progenitor":0,"state":"being_created","completed":0,"not_completed":0,"#,
            r#""clade_completed":0,"clade_not_completed":0,"clade_score":null}]}"#
        )
    );
}

#[test]
fn the_state_of_a_failed_harness_is_written_as_failed() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Failed)
        .unwrap();
    assert!(
        Report::of(&settings(), &genealogy)
            .json()
            .contains(r#""id":1,"progenitor":0,"state":"failed""#)
    );
}

#[test]
fn the_object_gives_no_lower_bound() {
    assert!(
        !Report::of(&settings(), &Genealogy::new())
            .json()
            .contains("lower_bound")
    );
}
