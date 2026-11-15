use cladenc::genealogy::{ExpansionOutcome, Genealogy};
use cladenc::policy::{Job, Policy};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn policy(seed: u64) -> Policy {
    Policy {
        budget: 1000,
        growth: 0.6,
        confidence: 99.0,
        tasks: tasks(),
        seed,
    }
}

fn with_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_expansion(0).unwrap();
    genealogy
        .end_expansion(child, ExpansionOutcome::Created)
        .unwrap();
    genealogy
}

fn generator(seed: u64, job: u64) -> ChaCha8Rng {
    let mut generator = ChaCha8Rng::seed_from_u64(seed);
    generator.set_stream(job);
    generator
}

#[test]
fn the_same_seed_and_job_number_give_the_same_job() {
    let genealogy = with_a_child();
    assert_eq!(
        policy(7).next_job(&genealogy, 3),
        policy(7).next_job(&genealogy, 3)
    );
}

#[test]
fn another_job_number_draws_from_another_stream() {
    let genealogy = with_a_child();
    let first = policy(7).next_job(&genealogy, 0);
    assert!((1..50).any(|job| policy(7).next_job(&genealogy, job) != first));
}

#[test]
fn another_seed_draws_other_numbers() {
    let genealogy = with_a_child();
    let first = policy(0).next_job(&genealogy, 0);
    assert!((1..50).any(|seed| policy(seed).next_job(&genealogy, 0) != first));
}

#[test]
fn job_k_draws_from_stream_k_of_the_seeded_generator_for_the_harness_and_then_the_task() {
    let mut generator = generator(7, 3);
    Beta::new(1.0, 1.0).unwrap().sample(&mut generator);
    let task = tasks()[generator.random_range(0..20)].clone();
    assert_eq!(
        policy(7).next_job(&Genealogy::new(), 3),
        Some(Job::Evaluation { harness: 0, task })
    );
}

#[test]
fn the_harnesses_are_drawn_for_in_ascending_id_order_and_the_largest_draw_is_evaluated() {
    let mut generator = generator(11, 5);
    let beta = Beta::new(1.0, 1.0).unwrap();
    let progenitor: f64 = beta.sample(&mut generator);
    let child: f64 = beta.sample(&mut generator);
    let task = tasks()[generator.random_range(0..20)].clone();
    assert_eq!(
        policy(11).next_job(&with_a_child(), 5),
        Some(Job::Evaluation {
            harness: u64::from(child > progenitor),
            task
        })
    );
}
