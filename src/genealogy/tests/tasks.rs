use crate::genealogy::{AttemptResult, CreationOutcome, Genealogy};

fn tasks(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn a_harness_given_no_task_has_every_task_of_the_list_left() {
    let list = tasks(&["a", "b"]);
    assert_eq!(
        Genealogy::new().harnesses()[0].tasks_not_given(&list),
        ["a", "b"]
    );
}

#[test]
fn the_tasks_not_given_leave_out_a_task_whose_attempt_has_not_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_leave_out_a_task_whose_attempt_has_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "a").unwrap();
    genealogy
        .end_attempt(0, "a", AttemptResult::Completed)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_keep_the_order_of_the_list() {
    let list = tasks(&["c", "a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["c", "b"]);
}

#[test]
fn a_harness_given_every_task_of_the_list_has_none_left() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_attempt(0, "a").unwrap();
    genealogy.start_attempt(0, "b").unwrap();
    assert_eq!(
        genealogy.harnesses()[0].tasks_not_given(&list),
        Vec::<&str>::new()
    );
}

#[test]
fn a_task_given_to_one_harness_is_not_given_to_another() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    genealogy.start_attempt(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[1].tasks_not_given(&list), ["a", "b"]);
}

fn attempted(genealogy: &mut Genealogy, task: &str, result: AttemptResult) {
    genealogy.start_attempt(0, task).unwrap();
    genealogy.end_attempt(0, task, result).unwrap();
}

#[test]
fn a_harness_with_a_finished_attempt_at_each_task_of_the_list_has_attempted_every_task() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    attempted(&mut genealogy, "a", AttemptResult::Completed);
    attempted(&mut genealogy, "b", AttemptResult::NotCompleted);
    assert!(genealogy.harnesses()[0].has_attempted_every(&list));
}

#[test]
fn a_harness_whose_attempt_ran_past_its_time_limit_has_attempted_that_task() {
    let list = tasks(&["a"]);
    let mut genealogy = Genealogy::new();
    attempted(&mut genealogy, "a", AttemptResult::TimedOut);
    assert!(genealogy.harnesses()[0].has_attempted_every(&list));
}

#[test]
fn a_harness_given_no_task_has_not_attempted_every_task() {
    let list = tasks(&["a", "b"]);
    assert!(!Genealogy::new().harnesses()[0].has_attempted_every(&list));
}

#[test]
fn a_harness_not_given_one_task_of_the_list_has_not_attempted_every_task() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    attempted(&mut genealogy, "a", AttemptResult::Completed);
    assert!(!genealogy.harnesses()[0].has_attempted_every(&list));
}

#[test]
fn a_harness_has_not_attempted_every_task_while_an_attempt_of_it_is_running() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    attempted(&mut genealogy, "a", AttemptResult::Completed);
    genealogy.start_attempt(0, "b").unwrap();
    assert!(!genealogy.harnesses()[0].has_attempted_every(&list));
}

#[test]
fn a_task_another_harness_attempted_does_not_count_for_a_harness() {
    let list = tasks(&["a"]);
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    attempted(&mut genealogy, "a", AttemptResult::Completed);
    assert!(!genealogy.harnesses()[1].has_attempted_every(&list));
}
