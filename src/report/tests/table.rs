use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::genealogy::{Cause, CreationOutcome, Genealogy, TrialResult};
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
            tasks: ["t1", "t2", "t3", "t4", "t5"].map(str::to_owned).to_vec(),
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_diversifications: 800,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 37.5,
        holdout_tasks: ["t1", "t2", "t3", "t4"]
            .map(|id| format!("benchmark/{id}"))
            .to_vec(),
        corroboration_tasks_percentage: 12.5,
        corroboration_tasks: vec!["benchmark/t4".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn trialled(genealogy: &mut Genealogy, task: &str, result: TrialResult) {
    genealogy.start_trial(0, task).unwrap();
    genealogy.end_trial(0, task, result).unwrap();
}

fn table(build: impl FnOnce(&mut Genealogy)) -> String {
    table_with(&[], build)
}

fn table_with(seconds: &[Seconds], build: impl FnOnce(&mut Genealogy)) -> String {
    let mut genealogy = settings().genealogy();
    build(&mut genealogy);
    Report::of(&settings(), &genealogy, seconds, Some(0)).table()
}

fn took(trials: f64, diversification: Option<f64>) -> Seconds {
    Seconds {
        trials,
        diversification,
    }
}

#[test]
fn the_table_holds_a_line_for_the_model_one_for_the_run_a_header_and_a_row_for_each_harness() {
    let table = table_with(&[took(4.5, None)], |genealogy| {
        trialled(genealogy, "benchmark/t1", TrialResult::Pass);
        trialled(genealogy, "benchmark/t4", TrialResult::Pass);
        trialled(genealogy, "benchmark/t5", TrialResult::Fail);
        genealogy.start_diversification(0).unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 1 of 800 diversifications, best harness 0\n\
         completed by at least one harness: selection tasks 1 of 3, corroboration tasks 1 of 1, non-holdout tasks 0 of 1\n\
         on the corroboration tasks: the best harness is the primogenitor, so there are no two harnesses to compare\n\
         experiments: conducted 0, the progenitor's child created in 0, the singleton control group harness created in 0, both created in 0\n\
         on the corroboration tasks of the experiments: no experiment created both the progenitor's child and the singleton control group harness, so there are no two harnesses to compare\n\
         \n\
         id  progenitor  experiment  state          selection passed  selection failed  selection inconclusive  corroboration passed  corroboration failed  corroboration inconclusive  non-holdout passed  non-holdout failed  non-holdout inconclusive  clade passed  clade failed  clade inconclusive  clade completion rate  trial seconds  diversification seconds\n\
         0   -           -           created        1                 0                 0                       1                     0                     0                           0                   1                   0                         1             0             0                   1.000                  4.5            -\n\
         1   0           -           being created  0                 0                 0                       0                     0                     0                           0                   0                   0                         0             0             0                   -                      0.0            -\n"
    );
}

#[test]
fn a_harness_whose_diversification_left_the_progenitor_s_clone_unchanged_is_shown_as_unchanged() {
    let table = table(|genealogy| {
        let child = genealogy.start_diversification(0).unwrap();
        genealogy
            .end_diversification(child, CreationOutcome::Unchanged)
            .unwrap();
    });
    assert!(
        table
            .lines()
            .last()
            .is_some_and(|row| row.starts_with("1   0           -           unchanged  0 "))
    );
}

#[test]
fn a_failed_harness_is_shown_as_failed() {
    let seconds = [took(0.0, None), took(0.0, Some(30.26))];
    let table = table_with(&seconds, |genealogy| {
        let child = genealogy.start_diversification(0).unwrap();
        genealogy
            .end_diversification(child, CreationOutcome::Failed)
            .unwrap();
    });
    assert_eq!(
        table,
        "model a-model at http://models.example/v1\n\
         started 1 of 800 diversifications, best harness 0\n\
         completed by at least one harness: selection tasks 0 of 3, corroboration tasks 0 of 1, non-holdout tasks 0 of 1\n\
         on the corroboration tasks: the best harness is the primogenitor, so there are no two harnesses to compare\n\
         experiments: conducted 0, the progenitor's child created in 0, the singleton control group harness created in 0, both created in 0\n\
         on the corroboration tasks of the experiments: no experiment created both the progenitor's child and the singleton control group harness, so there are no two harnesses to compare\n\
         \n\
         id  progenitor  experiment  state    selection passed  selection failed  selection inconclusive  corroboration passed  corroboration failed  corroboration inconclusive  non-holdout passed  non-holdout failed  non-holdout inconclusive  clade passed  clade failed  clade inconclusive  clade completion rate  trial seconds  diversification seconds\n\
         0   -           -           created  0                 0                 0                       0                     0                     0                           0                   0                   0                         0             0             0                   -                      0.0            -\n\
         1   0           -           failed   0                 0                 0                       0                     0                     0                           0                   0                   0                         0             0             0                   -                      0.0            30.3\n"
    );
}

#[test]
fn a_rate_is_printed_to_three_decimal_places() {
    let table = table(|genealogy| {
        trialled(genealogy, "benchmark/t1", TrialResult::Pass);
        trialled(genealogy, "benchmark/t2", TrialResult::Fail);
        trialled(genealogy, "benchmark/t3", TrialResult::Fail);
    });
    assert!(table.lines().nth(8).unwrap().contains("  0.333  "));
}

#[test]
fn the_most_diversifications_being_1_is_not_shown_as_several() {
    let settings = Settings {
        max_diversifications: 1,
        ..settings()
    };
    let table = Report::of(&settings, &Genealogy::new(), &[], Some(0)).table();
    assert_eq!(
        table.lines().nth(1),
        Some("started 0 of 1 diversification, best harness 0")
    );
}

#[test]
fn the_table_has_no_column_of_lower_bounds() {
    assert!(!table(|_| {}).contains("lower bound"));
}

#[test]
fn the_seconds_of_a_diversification_are_printed_to_one_decimal_place() {
    let seconds = [took(0.0, None), took(0.0, Some(30.26))];
    let table = table_with(&seconds, |genealogy| {
        genealogy.start_diversification(0).unwrap();
    });
    assert!(table.lines().nth(9).unwrap().ends_with("  30.3"));
}

#[test]
fn the_seconds_of_the_trials_of_a_harness_are_printed_to_one_decimal_place() {
    let table = table_with(&[took(12.345, None)], |_| {});
    assert!(table.lines().nth(8).unwrap().contains("  12.3  "));
}

#[test]
fn a_harness_for_which_no_seconds_are_given_is_shown_with_none_spent() {
    let table = table(|_| {});
    assert!(
        table
            .lines()
            .nth(8)
            .unwrap()
            .ends_with(&format!("  0.0{}-", " ".repeat(12)))
    );
}

#[test]
fn the_line_above_the_table_says_how_many_tasks_at_least_one_harness_completed() {
    assert_eq!(
        table(|_| {}).lines().nth(2),
        Some(
            "completed by at least one harness: selection tasks 0 of 3, corroboration tasks 0 \
             of 1, non-holdout tasks 0 of 1"
        )
    );
}

#[test]
fn an_empty_line_stands_between_what_is_said_of_the_directed_evolution_and_the_table() {
    assert_eq!(table(|_| {}).lines().nth(6), Some(""));
}

#[test]
fn a_corroboration_task_a_harness_completed_adds_nothing_to_its_clade_completion_rate() {
    let table = table(|genealogy| {
        trialled(genealogy, "benchmark/t1", TrialResult::Fail);
        trialled(genealogy, "benchmark/t4", TrialResult::Pass);
    });
    assert!(table.lines().nth(8).unwrap().contains("  0.000  "));
}

#[test]
fn the_header_names_the_columns_of_the_selection_tasks_ahead_of_those_of_the_corroboration_tasks() {
    assert!(table(|_| {}).lines().nth(7).unwrap().contains(
        "  selection passed  selection failed  selection inconclusive  corroboration passed  \
         corroboration failed  corroboration inconclusive  non-holdout passed  non-holdout \
         failed  non-holdout inconclusive  "
    ));
}

#[test]
fn an_inconclusive_trial_is_shown_in_a_column_of_its_own_and_not_as_failed() {
    let table = table(|genealogy| {
        trialled(
            genealogy,
            "benchmark/t1",
            TrialResult::Inconclusive(Cause::NoVerdict),
        );
    });
    assert!(table.lines().nth(8).unwrap().starts_with(
        "0   -           -           created  0                 0                 1  "
    ));
}

#[test]
fn an_inconclusive_trial_is_one_of_the_finished_trials_of_the_rate_that_is_printed() {
    let table = table(|genealogy| {
        trialled(genealogy, "benchmark/t1", TrialResult::Pass);
        trialled(
            genealogy,
            "benchmark/t2",
            TrialResult::Inconclusive(Cause::NoVerdict),
        );
    });
    assert!(table.lines().nth(8).unwrap().contains("  0.500  "));
}

#[test]
fn the_table_has_no_column_of_the_holdout_tasks_together() {
    assert!(!table(|_| {}).contains("  holdout"));
}

#[test]
fn the_table_names_no_score() {
    assert!(!table(|_| {}).contains("score"));
}

#[test]
fn the_header_names_the_completion_rate_of_the_clade_after_its_three_counts() {
    assert!(
        table(|_| {})
            .lines()
            .nth(7)
            .unwrap()
            .contains("  clade passed  clade failed  clade inconclusive  clade completion rate  ")
    );
}
