use std::path::PathBuf;

use crate::dataset::Dataset;
use crate::holdout::{self, HoldoutError, Percentages};

fn dataset(name: &str, tasks: usize) -> Dataset {
    Dataset {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task}")).collect(),
    }
}

fn refusal(datasets: &[Dataset], selection: f64, corroboration: f64) -> HoldoutError {
    holdout::draw(
        datasets,
        Percentages {
            selection,
            corroboration,
        },
        7,
    )
    .unwrap_err()
}

#[test]
fn a_dataset_of_two_tasks_is_refused_as_one_that_cannot_be_divided() {
    assert_eq!(
        refusal(&[dataset("bugs", 2)], 40.0, 40.0),
        HoldoutError::TooFewTasks {
            dataset: "bugs".to_owned()
        }
    );
}

#[test]
fn a_dataset_of_fewer_than_three_tasks_is_refused_whatever_the_percentages() {
    assert!((0..3).all(|tasks| {
        (1..100_u32).all(|percentage| {
            refusal(
                &[dataset("bugs", tasks)],
                f64::from(percentage),
                f64::from(100 - percentage),
            ) == HoldoutError::TooFewTasks {
                dataset: "bugs".to_owned(),
            }
        })
    }));
}

#[test]
fn a_selection_percentage_that_makes_no_task_of_a_dataset_a_selection_task_is_refused() {
    assert_eq!(
        refusal(&[dataset("bugs", 3)], 10.0, 40.0),
        HoldoutError::NoSelectionTask {
            dataset: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn a_corroboration_percentage_that_makes_no_task_of_a_dataset_a_corroboration_task_is_refused() {
    assert_eq!(
        refusal(&[dataset("bugs", 3)], 40.0, 10.0),
        HoldoutError::NoCorroborationTask {
            dataset: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn neither_a_selection_nor_a_corroboration_task_is_refused_for_the_selection_tasks() {
    assert_eq!(
        refusal(&[dataset("bugs", 3)], 10.0, 10.0),
        HoldoutError::NoSelectionTask {
            dataset: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn percentages_that_together_hold_out_every_task_of_a_dataset_are_refused() {
    assert_eq!(
        refusal(&[dataset("bugs", 3)], 50.0, 40.0),
        HoldoutError::NoNonHoldoutTask {
            dataset: "bugs".to_owned(),
            tasks: 3
        }
    );
}

#[test]
fn among_several_datasets_the_one_that_cannot_be_divided_is_named() {
    assert_eq!(
        refusal(&[dataset("bugs", 10), dataset("features", 3)], 10.0, 40.0),
        HoldoutError::NoSelectionTask {
            dataset: "features".to_owned(),
            tasks: 3
        }
    );
}
