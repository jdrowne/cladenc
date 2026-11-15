use std::path::PathBuf;

use cladenc::genealogy::{EvaluationOutcome, ExpansionOutcome, Genealogy};
use cladenc::journal::Settings;
use cladenc::report::Report;

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/progenitor"),
        tasks: vec!["t1".to_owned()],
        expander: PathBuf::from("/expander"),
        evaluator: PathBuf::from("/evaluator"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_expansions: 0,
        workers: 1,
        expansion_timeout: 60,
        evaluation_timeout: 60,
        seed: 7,
    }
}

#[test]
fn the_object_holds_the_values_of_the_run_and_of_each_harness() {
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "t1").unwrap();
    genealogy
        .end_evaluation(0, "t1", EvaluationOutcome::Success)
        .unwrap();
    genealogy.start_expansion(0).unwrap();
    assert_eq!(
        Report::of(&settings(), &genealogy).json(),
        concat!(
            r#"{"started":1,"budget":800,"best_harness":0,"harnesses":["#,
            r#"{"id":0,"parent":null,"state":"created","successes":1,"failures":0,"#,
            r#""clade_successes":1,"clade_failures":0,"clade_success_rate":1.0,"#,
            r#""lower_bound":0.09999999999999981},"#,
            r#"{"id":1,"parent":0,"state":"being_created","successes":0,"failures":0,"#,
            r#""clade_successes":0,"clade_failures":0,"clade_success_rate":null,"#,
            r#""lower_bound":null}]}"#
        )
    );
}

#[test]
fn the_state_of_a_failed_harness_is_written_as_failed() {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_expansion(0).unwrap();
    genealogy
        .end_expansion(child, ExpansionOutcome::Failed)
        .unwrap();
    assert!(
        Report::of(&settings(), &genealogy)
            .json()
            .contains(r#""id":1,"parent":0,"state":"failed""#)
    );
}
