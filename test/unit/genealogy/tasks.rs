use cladenc::genealogy::{Genealogy, ProcreationOutcome, Score};

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
fn the_tasks_not_given_leave_out_a_task_whose_assessment_has_not_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_leave_out_a_task_whose_assessment_has_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    genealogy.end_assessment(0, "a", Score::Success).unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_keep_the_order_of_the_list() {
    let list = tasks(&["c", "a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["c", "b"]);
}

#[test]
fn a_harness_given_every_task_of_the_list_has_none_left() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_assessment(0, "a").unwrap();
    genealogy.start_assessment(0, "b").unwrap();
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
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy.start_assessment(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[1].tasks_not_given(&list), ["a", "b"]);
}

fn scored(genealogy: &mut Genealogy, task: &str, score: Score) {
    genealogy.start_assessment(0, task).unwrap();
    genealogy.end_assessment(0, task, score).unwrap();
}

#[test]
fn a_harness_scored_on_each_task_of_the_list_has_been_scored_on_every_task() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    scored(&mut genealogy, "a", Score::Success);
    scored(&mut genealogy, "b", Score::Failure);
    assert!(genealogy.harnesses()[0].has_been_scored_on_every(&list));
}

#[test]
fn a_harness_whose_assessment_ran_past_its_time_limit_has_been_scored_on_that_task() {
    let list = tasks(&["a"]);
    let mut genealogy = Genealogy::new();
    scored(&mut genealogy, "a", Score::TimedOut);
    assert!(genealogy.harnesses()[0].has_been_scored_on_every(&list));
}

#[test]
fn a_harness_given_no_task_has_not_been_scored_on_every_task() {
    let list = tasks(&["a", "b"]);
    assert!(!Genealogy::new().harnesses()[0].has_been_scored_on_every(&list));
}

#[test]
fn a_harness_not_given_one_task_of_the_list_has_not_been_scored_on_every_task() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    scored(&mut genealogy, "a", Score::Success);
    assert!(!genealogy.harnesses()[0].has_been_scored_on_every(&list));
}

#[test]
fn a_harness_has_not_been_scored_on_every_task_while_an_assessment_of_it_is_running() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    scored(&mut genealogy, "a", Score::Success);
    genealogy.start_assessment(0, "b").unwrap();
    assert!(!genealogy.harnesses()[0].has_been_scored_on_every(&list));
}

#[test]
fn a_task_another_harness_was_scored_on_does_not_count_for_a_harness() {
    let list = tasks(&["a"]);
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    scored(&mut genealogy, "a", Score::Success);
    assert!(!genealogy.harnesses()[1].has_been_scored_on_every(&list));
}
