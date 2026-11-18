use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use test_inputs::{task, temporary_directory};

fn written_in(file: &str) -> String {
    let directory = temporary_directory();
    fs::read_to_string(task(directory.path(), "t1").join(file)).unwrap()
}

fn accepted(solved: Option<&str>) -> bool {
    let directory = temporary_directory();
    let grader = task(directory.path(), "t1").join("grader");
    if let Some(solved) = solved {
        fs::write(directory.path().join("solved"), solved).unwrap();
    }
    Command::new("sh")
        .arg(grader)
        .current_dir(directory.path())
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn a_task_is_the_directory_named_by_its_id() {
    let directory = temporary_directory();
    assert_eq!(task(directory.path(), "t1"), directory.path().join("t1"));
}

#[test]
fn the_task_specification_names_the_task() {
    assert_eq!(written_in("specification"), "Finish the work of t1.\n");
}

#[test]
fn the_workspace_holds_a_file_of_work_to_do() {
    assert_eq!(written_in("workspace/work.txt"), "to do\n");
}

#[test]
fn the_grader_looks_for_the_id_of_the_task_on_a_line_of_the_file_solved() {
    assert_eq!(written_in("grader"), "#!/bin/sh\ngrep -q -x -e t1 solved\n");
}

#[test]
fn the_grader_can_be_run_by_its_owner_and_its_group_and_no_one_else() {
    let directory = temporary_directory();
    let grader = task(directory.path(), "t1").join("grader");
    assert_eq!(
        fs::metadata(grader).unwrap().permissions().mode() & 0o777,
        0o750
    );
}

#[test]
fn the_grader_accepts_work_whose_file_solved_names_the_task() {
    assert!(accepted(Some("t1\n")));
}

#[test]
fn the_grader_accepts_work_whose_file_solved_names_the_task_among_others() {
    assert!(accepted(Some("t2\nt1\nt3\n")));
}

#[test]
fn the_grader_does_not_accept_work_whose_file_solved_names_other_tasks_alone() {
    assert!(!accepted(Some("t10\nt2\n")));
}

#[test]
fn the_grader_does_not_accept_work_that_has_no_file_solved() {
    assert!(!accepted(None));
}
