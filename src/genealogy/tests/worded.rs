use std::error::Error;

use crate::genealogy::GenealogyError;

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn a_progenitor_that_is_not_created_is_worded() {
    assert_eq!(
        worded(GenealogyError::ProgenitorNotCreated { progenitor: 3 }),
        "harness 3 cannot procreate because it is not a created harness"
    );
}

#[test]
fn a_primogenitor_that_does_not_await_its_build_is_worded() {
    assert_eq!(
        worded(GenealogyError::PrimogenitorNotAwaitingBuild),
        "the primogenitor is not waiting to be built"
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
fn a_harness_that_attempts_a_task_but_is_not_created_is_worded() {
    assert_eq!(
        worded(GenealogyError::HarnessNotCreated { harness: 3 }),
        "harness 3 cannot attempt a task because it is not a created harness"
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
fn an_attempt_that_is_not_running_is_worded() {
    assert_eq!(
        worded(GenealogyError::AttemptNotRunning {
            harness: 3,
            task: "t1".to_owned()
        }),
        "no attempt of harness 3 on task t1 is running"
    );
}
