use crate::holdout::HoldoutError;

#[test]
fn a_benchmark_of_one_task_is_worded() {
    assert_eq!(
        HoldoutError::OneTask {
            benchmark: "bugs".to_owned()
        }
        .to_string(),
        "the benchmark `bugs` holds one task, which cannot be divided"
    );
}

#[test]
fn a_benchmark_with_no_holdout_task_is_worded() {
    assert_eq!(
        HoldoutError::NoHoldoutTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` holds 3 tasks, and none of them would be held out"
    );
}

#[test]
fn a_benchmark_with_no_task_that_is_not_held_out_is_worded() {
    assert_eq!(
        HoldoutError::NoOtherTask {
            benchmark: "bugs".to_owned(),
            tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` holds 3 tasks, and every one of them would be held out"
    );
}

#[test]
fn a_benchmark_with_one_holdout_task_is_worded() {
    assert_eq!(
        HoldoutError::OneHoldoutTask {
            benchmark: "bugs".to_owned()
        }
        .to_string(),
        "the benchmark `bugs` has one holdout task, which cannot be divided"
    );
}

#[test]
fn a_benchmark_with_no_corroboration_task_is_worded() {
    assert_eq!(
        HoldoutError::NoCorroborationTask {
            benchmark: "bugs".to_owned(),
            holdout_tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` has 3 holdout tasks, and none of them would be a corroboration task"
    );
}

#[test]
fn a_benchmark_with_no_selection_task_is_worded() {
    assert_eq!(
        HoldoutError::NoSelectionTask {
            benchmark: "bugs".to_owned(),
            holdout_tasks: 3
        }
        .to_string(),
        "the benchmark `bugs` has 3 holdout tasks, and every one of them would be a \
         corroboration task"
    );
}
