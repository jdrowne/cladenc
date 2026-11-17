use cladenc::artificial_selection::{Action, ArtificialSelection};
use cladenc::genealogy::{AttemptResult, Genealogy, HarnessId, ProcreationOutcome};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

fn tasks() -> Vec<String> {
    (0..2).map(|number| format!("task-{number}")).collect()
}

fn artificial_selection(seed: u64) -> ArtificialSelection {
    ArtificialSelection {
        max_procreations: 4,
        growth: 1.0,
        tasks: tasks(),
        seed,
    }
}

fn with_two_harnesses_benchmarked_in_full() -> Genealogy {
    let mut genealogy = Genealogy::new();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, ProcreationOutcome::Created)
        .unwrap();
    for harness in [0, 1] {
        genealogy.start_attempt(harness, "task-0").unwrap();
        genealogy
            .end_attempt(harness, "task-0", AttemptResult::Completed)
            .unwrap();
        genealogy.start_attempt(harness, "task-1").unwrap();
        genealogy
            .end_attempt(harness, "task-1", AttemptResult::NotCompleted)
            .unwrap();
    }
    genealogy
}

fn generator(seed: u64, action: u64) -> ChaCha8Rng {
    let mut generator = ChaCha8Rng::seed_from_u64(seed);
    generator.set_stream(action);
    generator
}

fn progenitor_drawn(seed: u64, action: u64) -> HarnessId {
    let mut generator = generator(seed, action);
    let focus = 4.0 / 3.0;
    let primogenitor: f64 = Beta::new(focus * 3.0, focus * 3.0)
        .unwrap()
        .sample(&mut generator);
    let child: f64 = Beta::new(focus * 2.0, focus * 2.0)
        .unwrap()
        .sample(&mut generator);
    u64::from(child > primogenitor)
}

#[test]
fn the_same_seed_and_action_number_give_the_same_action() {
    let genealogy = with_two_harnesses_benchmarked_in_full();
    assert_eq!(
        artificial_selection(7).next_action(&genealogy, 3),
        artificial_selection(7).next_action(&genealogy, 3)
    );
}

#[test]
fn another_action_number_draws_from_another_stream() {
    let genealogy = with_two_harnesses_benchmarked_in_full();
    let first = artificial_selection(7).next_action(&genealogy, 0);
    assert!((1..50).any(|action| artificial_selection(7).next_action(&genealogy, action) != first));
}

#[test]
fn another_seed_draws_other_numbers() {
    let genealogy = with_two_harnesses_benchmarked_in_full();
    let first = artificial_selection(0).next_action(&genealogy, 0);
    assert!((1..50).any(|seed| artificial_selection(seed).next_action(&genealogy, 0) != first));
}

#[test]
fn action_k_draws_from_stream_k_of_the_seeded_generator_for_each_harness_in_ascending_id_order() {
    let genealogy = with_two_harnesses_benchmarked_in_full();
    let drawn: Vec<Option<Action>> = (0..20)
        .map(|action| artificial_selection(7).next_action(&genealogy, action))
        .collect();
    let expected: Vec<Option<Action>> = (0..20)
        .map(|action| {
            Some(Action::Procreation {
                progenitor: progenitor_drawn(7, action),
            })
        })
        .collect();
    assert_eq!(drawn, expected);
}

#[test]
fn the_draws_are_sharpened_by_the_most_procreations_over_those_not_yet_started() {
    let genealogy = with_two_harnesses_benchmarked_in_full();
    let drawn: Vec<Option<Action>> = (0..20)
        .map(|seed| artificial_selection(seed).next_action(&genealogy, 5))
        .collect();
    let expected: Vec<Option<Action>> = (0..20)
        .map(|seed| {
            Some(Action::Procreation {
                progenitor: progenitor_drawn(seed, 5),
            })
        })
        .collect();
    assert_eq!(drawn, expected);
}

#[test]
fn an_attempt_is_the_same_whatever_the_seed() {
    let first = artificial_selection(0).next_action(&Genealogy::new(), 0);
    assert!(
        (1..50).all(|seed| artificial_selection(seed).next_action(&Genealogy::new(), 0) == first)
    );
}
