use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{Genealogy, ProcreationOutcome};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

fn tasks() -> Vec<String> {
    (0..20).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        budget: 1000,
        growth: 0.6,
        confidence: 99.0,
        tasks: tasks(),
        seed,
    }
}

fn with_a_child() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    genealogy
}

fn generator(seed: u64, action: u64) -> ChaCha8Rng {
    let mut generator = ChaCha8Rng::seed_from_u64(seed);
    generator.set_stream(action);
    generator
}

#[test]
fn the_same_seed_and_action_number_give_the_same_action() {
    let genealogy = with_a_child();
    assert_eq!(
        artificial_selection(7).next_action(&genealogy, 3),
        artificial_selection(7).next_action(&genealogy, 3)
    );
}

#[test]
fn another_action_number_draws_from_another_stream() {
    let genealogy = with_a_child();
    let first = artificial_selection(7).next_action(&genealogy, 0);
    assert!((1..50).any(|action| artificial_selection(7).next_action(&genealogy, action) != first));
}

#[test]
fn another_seed_draws_other_numbers() {
    let genealogy = with_a_child();
    let first = artificial_selection(0).next_action(&genealogy, 0);
    assert!((1..50).any(|seed| artificial_selection(seed).next_action(&genealogy, 0) != first));
}

#[test]
fn action_k_draws_from_stream_k_of_the_seeded_generator_for_the_harness_and_then_the_task() {
    let mut generator = generator(7, 3);
    Beta::new(1.0, 1.0).unwrap().sample(&mut generator);
    let task = tasks()[generator.random_range(0..20)].clone();
    assert_eq!(
        artificial_selection(7).next_action(&Genealogy::new(), 3),
        Some(Action::Assessment { harness: 0, task })
    );
}

#[test]
fn the_harnesses_are_drawn_for_in_ascending_id_order_and_the_largest_draw_is_assessed() {
    let mut generator = generator(11, 5);
    let beta = Beta::new(1.0, 1.0).unwrap();
    let primogenitor: f64 = beta.sample(&mut generator);
    let child: f64 = beta.sample(&mut generator);
    let task = tasks()[generator.random_range(0..20)].clone();
    assert_eq!(
        artificial_selection(11).next_action(&with_a_child(), 5),
        Some(Action::Assessment {
            harness: u64::from(child > primogenitor),
            task
        })
    );
}
