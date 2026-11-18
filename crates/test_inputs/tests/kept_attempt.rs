use std::fs;

use test_inputs::{kept_attempt, temporary_directory};

#[test]
fn an_attempt_is_found_among_the_attempts_of_its_harness() {
    let outputs = temporary_directory();
    let attempt = outputs.path().join("attempts/3/benchmark/t1");
    fs::create_dir_all(&attempt).unwrap();
    assert_eq!(kept_attempt(outputs.path(), 3, "benchmark/t1"), attempt);
}

#[test]
fn an_attempt_at_a_holdout_task_is_found_under_holdout() {
    let outputs = temporary_directory();
    let attempt = outputs.path().join("holdout/3/benchmark/t1");
    fs::create_dir_all(&attempt).unwrap();
    assert_eq!(kept_attempt(outputs.path(), 3, "benchmark/t1"), attempt);
}

#[test]
fn the_attempt_of_another_harness_at_the_task_is_not_taken_for_it() {
    let outputs = temporary_directory();
    let attempt = outputs.path().join("holdout/3/benchmark/t1");
    fs::create_dir_all(outputs.path().join("attempts/2/benchmark/t1")).unwrap();
    fs::create_dir_all(&attempt).unwrap();
    assert_eq!(kept_attempt(outputs.path(), 3, "benchmark/t1"), attempt);
}

#[test]
fn the_attempt_of_the_harness_at_another_task_is_not_taken_for_it() {
    let outputs = temporary_directory();
    let attempt = outputs.path().join("holdout/3/benchmark/t1");
    fs::create_dir_all(outputs.path().join("attempts/3/benchmark/t2")).unwrap();
    fs::create_dir_all(&attempt).unwrap();
    assert_eq!(kept_attempt(outputs.path(), 3, "benchmark/t1"), attempt);
}

#[test]
#[should_panic(expected = "the attempt is kept in the outputs directory")]
fn an_attempt_that_was_not_kept_is_not_found() {
    let outputs = temporary_directory();
    kept_attempt(outputs.path(), 3, "benchmark/t1");
}
