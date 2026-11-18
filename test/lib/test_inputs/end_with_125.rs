use std::fs;
use std::path::{Path, PathBuf};

use test_inputs::{
    benchmark, end_with_125_at, end_with_125_no_more, end_with_125_on, end_with_125_on_no_more,
    primogenitor, temporary_directory,
};

fn harness(outputs: &Path, number: u64, marked: bool) -> PathBuf {
    let harness = outputs.join("harnesses").join(number.to_string());
    fs::create_dir_all(&harness).unwrap();
    fs::write(harness.join("solved"), "t1\n").unwrap();
    if marked {
        fs::write(harness.join("end-with-125-at"), "1\n").unwrap();
    }
    harness
}

fn kept_tasks_marked(outputs: &Path, tasks: &[&str]) -> PathBuf {
    let kept = benchmark(&outputs.join("benchmarks"), tasks);
    for task in tasks {
        end_with_125_on(&outputs.join("benchmarks"), task);
    }
    kept
}

#[test]
fn the_marker_in_a_primogenitor_holds_the_count_of_solved_tasks_to_end_with_125_at() {
    let directory = temporary_directory();
    let made = primogenitor(directory.path());
    end_with_125_at(&made, 2);
    assert_eq!(
        fs::read_to_string(made.join("end-with-125-at")).unwrap(),
        "2\n"
    );
}

#[test]
fn the_marker_of_every_harness_is_removed() {
    let outputs = temporary_directory();
    let harnesses = [0, 1].map(|number| harness(outputs.path(), number, true));
    end_with_125_no_more(outputs.path());
    assert_eq!(
        harnesses.map(|harness| harness.join("end-with-125-at").exists()),
        [false, false]
    );
}

#[test]
fn a_harness_that_has_no_marker_is_passed_over() {
    let outputs = temporary_directory();
    let marked = harness(outputs.path(), 0, true);
    harness(outputs.path(), 1, false);
    end_with_125_no_more(outputs.path());
    assert!(!marked.join("end-with-125-at").exists());
}

#[test]
fn what_else_a_harness_holds_is_kept_when_its_marker_is_removed() {
    let outputs = temporary_directory();
    let marked = harness(outputs.path(), 0, true);
    end_with_125_no_more(outputs.path());
    assert_eq!(fs::read_to_string(marked.join("solved")).unwrap(), "t1\n");
}

#[test]
fn the_marker_on_a_task_is_an_empty_file_in_its_workspace() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1"]);
    end_with_125_on(directory.path(), "t1");
    assert_eq!(
        fs::read_to_string(made.join("t1/workspace/end-with-125")).unwrap(),
        ""
    );
}

#[test]
fn the_marker_is_put_on_the_task_named_and_on_no_other() {
    let directory = temporary_directory();
    let made = benchmark(directory.path(), &["t1", "t2"]);
    end_with_125_on(directory.path(), "t1");
    assert!(!made.join("t2/workspace/end-with-125").exists());
}

#[test]
fn the_marker_on_a_task_kept_in_an_outputs_directory_is_removed() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(!kept.join("t1/workspace/end-with-125").exists());
}

#[test]
fn the_marker_on_another_kept_task_stays() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1", "t2"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(kept.join("t2/workspace/end-with-125").exists());
}

#[test]
fn the_work_to_do_stays_in_the_workspace_when_the_marker_on_a_task_is_removed() {
    let outputs = temporary_directory();
    let kept = kept_tasks_marked(outputs.path(), &["t1"]);
    end_with_125_on_no_more(outputs.path(), "t1");
    assert!(kept.join("t1/workspace/work.txt").exists());
}
