use cladenc::genealogy::{EvaluationOutcome, Genealogy, MutationOutcome};
use cladenc::policy::Policy;

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn policy(budget: u64, growth: f64) -> Policy {
    Policy {
        budget,
        growth,
        confidence: 99.0,
        tasks: tasks(),
        seed: 0,
    }
}

fn with_every_task_given() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_mutation(0).unwrap();
    genealogy
        .end_mutation(child, MutationOutcome::Failed)
        .unwrap();
    for task in tasks() {
        genealogy.start_evaluation(0, &task).unwrap();
        genealogy
            .end_evaluation(0, &task, EvaluationOutcome::Success)
            .unwrap();
    }
    genealogy
}

#[test]
fn there_is_no_next_job_when_no_mutation_is_due_and_every_task_has_been_given() {
    assert_eq!(policy(100, 0.0).next_job(&with_every_task_given(), 0), None);
}

#[test]
fn there_is_no_next_job_once_the_budget_is_spent() {
    assert_eq!(policy(2, 1.0).next_job(&with_every_task_given(), 0), None);
}
