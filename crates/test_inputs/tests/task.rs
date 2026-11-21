use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use test_inputs::{task, temporary_directory};

fn written_in(file: &str) -> String {
    let directory = temporary_directory();
    fs::read_to_string(task(directory.path(), "t1").join(file)).unwrap()
}

fn scored(solved: Option<&str>) -> String {
    let directory = temporary_directory();
    let verifier = directory.path().join("verifier");
    let grader = fs::read_to_string(task(directory.path(), "t1").join("tests/test.sh")).unwrap();
    fs::create_dir(&verifier).unwrap();
    if let Some(solved) = solved {
        fs::write(directory.path().join("solved"), solved).unwrap();
    }
    Command::new("sh")
        .arg("-c")
        .arg(grader.replace("/logs/verifier", &verifier.to_string_lossy()))
        .current_dir(directory.path())
        .output()
        .unwrap();
    fs::read_to_string(verifier.join("reward.txt")).unwrap()
}

#[test]
fn a_task_is_the_directory_named_by_its_id() {
    let directory = temporary_directory();
    assert_eq!(task(directory.path(), "t1"), directory.path().join("t1"));
}

#[test]
fn the_task_specification_names_the_task() {
    assert_eq!(written_in("instruction.md"), "Finish the work of t1.\n");
}

#[test]
fn the_workspace_holds_a_file_of_work_to_do() {
    assert_eq!(written_in("workspace/work.txt"), "to do\n");
}

#[test]
fn the_environment_is_built_from_the_test_image() {
    assert_eq!(
        written_in("environment/Dockerfile"),
        "FROM debian:13.7-slim\n"
    );
}

#[test]
fn the_grader_writes_a_score_of_1_when_a_line_of_the_file_solved_is_the_id_of_the_task() {
    assert_eq!(
        written_in("tests/test.sh"),
        "#!/bin/sh\nif grep -q -x -e t1 solved; then echo 1; else echo 0; fi \
         > /logs/verifier/reward.txt\n"
    );
}

#[test]
fn the_grader_can_be_run_by_its_owner_and_its_group_and_no_one_else() {
    let directory = temporary_directory();
    let grader = task(directory.path(), "t1").join("tests/test.sh");
    assert_eq!(
        fs::metadata(grader).unwrap().permissions().mode() & 0o777,
        0o750
    );
}

#[test]
fn the_grader_scores_1_for_work_whose_file_solved_names_the_task() {
    assert_eq!(scored(Some("t1\n")), "1\n");
}

#[test]
fn the_grader_scores_1_for_work_whose_file_solved_names_the_task_among_others() {
    assert_eq!(scored(Some("t2\nt1\nt3\n")), "1\n");
}

#[test]
fn the_grader_scores_0_for_work_whose_file_solved_names_other_tasks_alone() {
    assert_eq!(scored(Some("t10\nt2\n")), "0\n");
}

#[test]
fn the_grader_scores_0_for_work_that_has_no_file_solved() {
    assert_eq!(scored(None), "0\n");
}
