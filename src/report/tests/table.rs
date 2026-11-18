use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{AttemptResult, Genealogy, ProcreationOutcome};
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
        holdout_tasks: Vec::new(),
        seed: 7,
    }
}

fn attempted(genealogy: &mut Genealogy, task: &str, result: AttemptResult) {
    genealogy.start_attempt(0, task).unwrap();
    genealogy.end_attempt(0, task, result).unwrap();
}

fn table(build: impl FnOnce(&mut Genealogy)) -> String {
    table_with(&[], build)
}

fn table_with(seconds: &[Seconds], build: impl FnOnce(&mut Genealogy)) -> String {
    let mut genealogy =
        Genealogy::new().holding_out(&["t1".to_owned(), "t2".to_owned(), "t3".to_owned()]);
    build(&mut genealogy);
    Report::of(&settings(), &genealogy, seconds).table()
}

fn took(attempts: f64, procreation: Option<f64>) -> Seconds {
    Seconds {
        attempts,
        procreation,
    }
}

#[test]
fn the_table_holds_a_line_for_the_model_one_for_the_run_a_header_and_a_row_for_each_harness() {
    let table = table_with(&[took(4.5, None)], |genealogy| {
        attempted(genealogy, "t1", AttemptResult::Completed);
        attempted(genealogy, "t4", AttemptResult::NotCompleted);
        genealogy.start_procreation(0).unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 1 of 800 procreations, best harness 0\n\
         completed by at least one harness: holdout tasks 0 of 0, other tasks 0 of 3\n\
         \n\
         id  progenitor  state          holdout completed  holdout not completed  other completed  other not completed  clade completed  clade not completed  clade score  attempt seconds  procreation seconds\n\
         0   -           created        1                  0                      0                1                    1                0                    1.000        4.5              -\n\
         1   0           being created  0                  0                      0                0                    0                0                    -            0.0              -\n"
    );
}

#[test]
fn a_failed_harness_is_shown_as_failed() {
    let seconds = [took(0.0, None), took(0.0, Some(30.26))];
    let table = table_with(&seconds, |genealogy| {
        let child = genealogy.start_procreation(0).unwrap();
        genealogy
            .end_procreation(child, ProcreationOutcome::Failed)
            .unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 1 of 800 procreations, best harness 0\n\
         completed by at least one harness: holdout tasks 0 of 0, other tasks 0 of 3\n\
         \n\
         id  progenitor  state    holdout completed  holdout not completed  other completed  other not completed  clade completed  clade not completed  clade score  attempt seconds  procreation seconds\n\
         0   -           created  0                  0                      0                0                    0                0                    -            0.0              -\n\
         1   0           failed   0                  0                      0                0                    0                0                    -            0.0              30.3\n"
    );
}

#[test]
fn a_rate_is_printed_to_three_decimal_places() {
    let table = table(|genealogy| {
        attempted(genealogy, "t1", AttemptResult::Completed);
        attempted(genealogy, "t2", AttemptResult::NotCompleted);
        attempted(genealogy, "t3", AttemptResult::NotCompleted);
    });
    assert!(table.lines().nth(5).unwrap().contains("  0.333  "));
}

#[test]
fn the_most_procreations_being_1_is_not_shown_as_several() {
    let settings = Settings {
        max_procreations: 1,
        ..settings()
    };
    let table = Report::of(&settings, &Genealogy::new(), &[]).table();
    assert_eq!(
        table.lines().nth(1),
        Some("started 0 of 1 procreation, best harness 0")
    );
}

#[test]
fn the_table_has_no_column_of_lower_bounds() {
    assert!(!table(|_| {}).contains("lower bound"));
}

#[test]
fn the_seconds_of_a_procreation_are_printed_to_one_decimal_place() {
    let seconds = [took(0.0, None), took(0.0, Some(30.26))];
    let table = table_with(&seconds, |genealogy| {
        genealogy.start_procreation(0).unwrap();
    });
    assert!(table.lines().nth(6).unwrap().ends_with("  30.3"));
}

#[test]
fn the_seconds_of_the_attempts_of_a_harness_are_printed_to_one_decimal_place() {
    let table = table_with(&[took(12.345, None)], |_| {});
    assert!(table.lines().nth(5).unwrap().contains("  12.3  "));
}

#[test]
fn a_harness_for_which_no_seconds_are_given_is_shown_with_none_spent() {
    let table = table(|_| {});
    assert!(
        table
            .lines()
            .nth(5)
            .unwrap()
            .ends_with(&format!("  0.0{}-", " ".repeat(14)))
    );
}

#[test]
fn the_line_above_the_table_says_how_many_tasks_at_least_one_harness_completed() {
    assert_eq!(
        table(|_| {}).lines().nth(2),
        Some("completed by at least one harness: holdout tasks 0 of 0, other tasks 0 of 3")
    );
}

#[test]
fn an_empty_line_stands_between_what_is_said_of_the_directed_evolution_and_the_table() {
    assert_eq!(table(|_| {}).lines().nth(3), Some(""));
}
