use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::holdout::{self, Drawn, Percentages};
use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

const PERCENTAGES: Percentages = Percentages {
    selection: 30.0,
    corroboration: 20.0,
};

fn benchmark(name: &str, tasks: usize) -> Benchmark {
    Benchmark {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task:02}")).collect(),
    }
}

fn benchmarks() -> [Benchmark; 2] {
    [benchmark("bugs", 20), benchmark("features", 8)]
}

fn draw(seed: u64) -> Drawn {
    holdout::draw(&benchmarks(), PERCENTAGES, seed).unwrap()
}

fn held_out(seed: u64) -> Vec<String> {
    draw(seed).holdout_tasks
}

fn drawn(seed: u64) -> Vec<String> {
    draw(seed).corroboration_tasks
}

fn of(name: &str, tasks: Vec<String>) -> Vec<String> {
    tasks
        .into_iter()
        .filter(|task| task.starts_with(name))
        .collect()
}

fn selection_tasks_of(name: &str, seed: u64) -> usize {
    let corroboration_tasks = drawn(seed);
    of(name, held_out(seed))
        .iter()
        .filter(|task| !corroboration_tasks.contains(task))
        .count()
}

#[test]
fn the_corroboration_percentage_is_taken_of_the_tasks_of_each_benchmark_by_itself() {
    assert_eq!(
        (of("bugs/", drawn(7)).len(), of("features/", drawn(7)).len()),
        (4, 2)
    );
}

#[test]
fn the_selection_percentage_is_taken_of_the_tasks_of_each_benchmark_by_itself() {
    assert_eq!(
        (
            selection_tasks_of("bugs/", 7),
            selection_tasks_of("features/", 7)
        ),
        (6, 2)
    );
}

#[test]
fn every_corroboration_task_is_a_holdout_task() {
    assert!(drawn(7).iter().all(|task| held_out(7).contains(task)));
}

#[test]
fn no_task_is_drawn_twice() {
    assert_eq!(drawn(7).into_iter().collect::<BTreeSet<_>>().len(), 6);
}

#[test]
fn the_corroboration_tasks_are_in_the_order_of_the_benchmarks_and_of_their_tasks() {
    let mut in_order = drawn(7);
    in_order.sort();
    assert_eq!(drawn(7), in_order);
}

#[test]
fn the_same_seed_draws_the_same_tasks() {
    assert_eq!(drawn(7), drawn(7));
}

#[test]
fn another_seed_draws_other_tasks() {
    assert!((1..50).any(|seed| drawn(seed) != drawn(0)));
}

#[test]
fn every_task_is_a_corroboration_task_under_one_seed_or_another() {
    assert_eq!(
        (0..200).flat_map(drawn).collect::<BTreeSet<_>>(),
        (0..200).flat_map(held_out).collect::<BTreeSet<_>>()
    );
}

#[test]
fn the_tasks_are_drawn_from_the_last_stream_but_one_of_the_seeded_generator() {
    let mut generator = ChaCha8Rng::seed_from_u64(7);
    generator.set_stream(u64::MAX - 1);
    let mut places = index::sample(&mut generator, 10, 4).into_vec();
    places.sort_unstable();
    let holdout_tasks = of("bugs/", held_out(7));
    assert_eq!(
        of("bugs/", drawn(7)),
        places
            .iter()
            .map(|place| holdout_tasks[*place].clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_benchmark_of_three_tasks_can_have_one_corroboration_task() {
    assert_eq!(
        holdout::draw(
            &[benchmark("bugs", 3)],
            Percentages {
                selection: 30.0,
                corroboration: 30.0
            },
            7
        )
        .unwrap()
        .corroboration_tasks
        .len(),
        1
    );
}
