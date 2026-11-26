use std::fs;

use test_inputs::{kept_trial, temporary_directory};

#[test]
fn a_trial_is_found_among_the_trials_of_its_harness() {
    let outputs = temporary_directory();
    let trial = outputs.path().join("trials/3/dataset/t1");
    fs::create_dir_all(&trial).unwrap();
    assert_eq!(kept_trial(outputs.path(), 3, "dataset/t1"), trial);
}

#[test]
fn a_trial_on_a_holdout_task_is_found_under_holdout() {
    let outputs = temporary_directory();
    let trial = outputs.path().join("holdout/3/dataset/t1");
    fs::create_dir_all(&trial).unwrap();
    assert_eq!(kept_trial(outputs.path(), 3, "dataset/t1"), trial);
}

#[test]
fn the_trial_of_another_harness_at_the_task_is_not_taken_for_it() {
    let outputs = temporary_directory();
    let trial = outputs.path().join("holdout/3/dataset/t1");
    fs::create_dir_all(outputs.path().join("trials/2/dataset/t1")).unwrap();
    fs::create_dir_all(&trial).unwrap();
    assert_eq!(kept_trial(outputs.path(), 3, "dataset/t1"), trial);
}

#[test]
fn the_trial_of_the_harness_at_another_task_is_not_taken_for_it() {
    let outputs = temporary_directory();
    let trial = outputs.path().join("holdout/3/dataset/t1");
    fs::create_dir_all(outputs.path().join("trials/3/dataset/t2")).unwrap();
    fs::create_dir_all(&trial).unwrap();
    assert_eq!(kept_trial(outputs.path(), 3, "dataset/t1"), trial);
}

#[test]
#[should_panic(expected = "the trial is kept in the outputs directory")]
fn a_trial_that_was_not_kept_is_not_found() {
    let outputs = temporary_directory();
    kept_trial(outputs.path(), 3, "dataset/t1");
}
