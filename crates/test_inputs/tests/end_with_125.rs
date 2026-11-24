use std::fs;
use std::path::{Path, PathBuf};

use test_inputs::{benchmark, end_with_125_on, end_with_125_on_no_more, temporary_directory};

fn kept_tasks_marked(outputs: &Path, tasks: &[&str]) -> PathBuf {
    let kept = benchmark(&outputs.join("benchmarks"), tasks);
    for task in tasks {
        end_with_125_on(&outputs.join("benchmarks"), task);
    }
    kept
}

#[test]
fn the_marker_on_a_task_is_an_empty_file_in_the_workspace_of_its_environment() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1"]);
    end_with_125_on(directory.path(), "t1");
    assert_eq!(
        fs::read_to_string(made.join("t1/environment/workspace/end-with-125")).unwrap(),
        ""
    );
}

#[test]
fn the_marker_is_put_on_the_task_named_and_on_no_other() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1", "t2"]);
    end_with_125_on(directory.path(), "t1");
    assert!(!made.join("t2/environment/workspace/end-with-125").exists());
}

#[test]
fn the_marker_on_a_task_kept_in_an_outputs_directory_is_removed() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(!kept.join("t1/environment/workspace/end-with-125").exists());
}

#[test]
fn the_marker_on_another_kept_task_stays() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1", "t2"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(kept.join("t2/environment/workspace/end-with-125").exists());
}

#[test]
fn the_work_to_do_stays_in_the_workspace_of_the_environment_when_the_marker_is_removed() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(kept.join("t1/environment/workspace/work.txt").exists());
}
