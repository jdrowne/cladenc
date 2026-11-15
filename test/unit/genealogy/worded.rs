use std::error::Error;

use cladenc::genealogy::GenealogyError;

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn a_parent_that_is_not_created_is_worded() {
    assert_eq!(
        worded(GenealogyError::ParentNotCreated { parent: 3 }),
        "harness 3 cannot procreate because it is not a created harness"
    );
}

#[test]
fn a_child_that_is_not_being_created_is_worded() {
    assert_eq!(
        worded(GenealogyError::ChildNotBeingCreated { child: 3 }),
        "harness 3 is not being created"
    );
}

#[test]
fn an_evaluated_harness_that_is_not_created_is_worded() {
    assert_eq!(
        worded(GenealogyError::HarnessNotCreated { harness: 3 }),
        "harness 3 cannot be evaluated because it is not a created harness"
    );
}

#[test]
fn a_task_given_twice_is_worded() {
    assert_eq!(
        worded(GenealogyError::TaskAlreadyGiven {
            harness: 3,
            task: "t1".to_owned()
        }),
        "harness 3 has already been given task t1"
    );
}

#[test]
fn an_evaluation_that_is_not_running_is_worded() {
    assert_eq!(
        worded(GenealogyError::EvaluationNotRunning {
            harness: 3,
            task: "t1".to_owned()
        }),
        "no evaluation of harness 3 on task t1 is running"
    );
}
