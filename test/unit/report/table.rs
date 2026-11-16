use std::path::PathBuf;

use cladenc::benchmark::Benchmark;
use cladenc::genealogy::{AssessmentOutcome, Genealogy, ProcreationOutcome};
use cladenc::journal::Settings;
use cladenc::report::Report;

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/progenitor"),
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
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 0,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        assessment_timeout: 60,
        seed: 7,
    }
}

fn assessed(genealogy: &mut Genealogy, task: &str, outcome: AssessmentOutcome) {
    genealogy.start_assessment(0, task).unwrap();
    genealogy.end_assessment(0, task, outcome).unwrap();
}

fn table(build: impl FnOnce(&mut Genealogy)) -> String {
    let mut genealogy = Genealogy::new();
    build(&mut genealogy);
    Report::of(&settings(), &genealogy).table()
}

#[test]
fn the_table_holds_a_line_for_the_model_one_for_the_run_a_header_and_a_row_for_each_harness() {
    let table = table(|genealogy| {
        assessed(genealogy, "t1", AssessmentOutcome::Success);
        genealogy.start_procreation(0).unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 1 of 800, best harness 0\n\
         \n\
         id  parent  state          successes  failures  clade successes  clade failures  clade success rate  lower bound\n\
         0   -       created        1          0         1                0               1.000               0.100\n\
         1   0       being created  0          0         0                0               -                   -\n"
    );
}

#[test]
fn a_failed_harness_is_shown_as_failed_with_no_lower_bound() {
    let table = table(|genealogy| {
        let child = genealogy.start_procreation(0).unwrap();
        genealogy
            .end_procreation(child, ProcreationOutcome::Failed)
            .unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 0 of 800, best harness 0\n\
         \n\
         id  parent  state    successes  failures  clade successes  clade failures  clade success rate  lower bound\n\
         0   -       created  0          0         0                0               -                   0.010\n\
         1   0       failed   0          0         0                0               -                   -\n"
    );
}

#[test]
fn a_rate_is_printed_to_three_decimal_places() {
    let table = table(|genealogy| {
        assessed(genealogy, "t1", AssessmentOutcome::Success);
        assessed(genealogy, "t2", AssessmentOutcome::Failure);
        assessed(genealogy, "t3", AssessmentOutcome::Failure);
    });
    assert!(table.lines().nth(4).unwrap().contains("  0.333  "));
}
