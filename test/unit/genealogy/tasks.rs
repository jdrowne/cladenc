use cladenc::genealogy::{EvaluationOutcome, Genealogy, ProcreationOutcome};

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
fn the_tasks_not_given_leave_out_a_task_whose_evaluation_has_not_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_leave_out_a_task_whose_evaluation_has_ended() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    genealogy
        .end_evaluation(0, "a", EvaluationOutcome::Success)
        .unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["b"]);
}

#[test]
fn the_tasks_not_given_keep_the_order_of_the_list() {
    let list = tasks(&["c", "a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[0].tasks_not_given(&list), ["c", "b"]);
}

#[test]
fn a_harness_given_every_task_of_the_list_has_none_left() {
    let list = tasks(&["a", "b"]);
    let mut genealogy = Genealogy::new();
    genealogy.start_evaluation(0, "a").unwrap();
    genealogy.start_evaluation(0, "b").unwrap();
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
    genealogy.start_evaluation(0, "a").unwrap();
    assert_eq!(genealogy.harnesses()[1].tasks_not_given(&list), ["a", "b"]);
}
