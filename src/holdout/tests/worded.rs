use crate::holdout::HoldoutError;

#[test]
fn a_benchmark_of_fewer_than_three_tasks_is_worded() {
    assert_eq!(
        HoldoutError::TooFewTasks {
            benchmark: "bugs".to_owned()
        }
        .to_string(),
        "the benchmark `bugs` holds fewer than three tasks, which cannot be divided into a \
         selection task, a corroboration task and a non-holdout task"
    );
}

#[test]
fn a_benchmark_with_no_selection_task_is_worded() {
    assert_eq!(
        HoldoutError::NoSelectionTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` holds 3 tasks, and none of them would be a selection task"
    );
}

#[test]
fn a_benchmark_with_no_corroboration_task_is_worded() {
    assert_eq!(
        HoldoutError::NoCorroborationTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` holds 3 tasks, and none of them would be a corroboration task"
    );
}

#[test]
fn a_benchmark_with_no_non_holdout_task_is_worded() {
    assert_eq!(
        HoldoutError::NoNonHoldoutTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` holds 3 tasks, and every one of them would be held out"
    );
}
