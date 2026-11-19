use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::holdout;
use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

fn benchmark(name: &str, tasks: usize) -> Benchmark {
    Benchmark {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task:02}")).collect(),
    }
}

fn benchmarks() -> [Benchmark; 2] {
    [benchmark("bugs", 20), benchmark("features", 8)]
}

fn held_out(seed: u64) -> Vec<String> {
    holdout::draw(&benchmarks(), 50.0, seed).unwrap()
}

fn drawn_out_of(holdout_tasks: &[String], seed: u64) -> Vec<String> {
    holdout::draw_corroboration(&benchmarks(), holdout_tasks, 40.0, seed).unwrap()
}

fn drawn(seed: u64) -> Vec<String> {
    drawn_out_of(&held_out(seed), seed)
}

fn of(name: &str, tasks: Vec<String>) -> Vec<String> {
    tasks
        .into_iter()
        .filter(|task| task.starts_with(name))
        .collect()
}

#[test]
fn the_percentage_is_taken_of_the_holdout_tasks_of_each_benchmark_by_itself() {
    assert_eq!(
        (of("bugs/", drawn(7)).len(), of("features/", drawn(7)).len()),
        (4, 2)
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
fn another_seed_draws_other_tasks_out_of_the_same_holdout_tasks() {
    let holdout_tasks = held_out(0);
    assert!(
        (1..50).any(|seed| drawn_out_of(&holdout_tasks, seed) != drawn_out_of(&holdout_tasks, 0))
    );
}

#[test]
fn every_holdout_task_is_a_corroboration_task_under_one_seed_or_another() {
    let holdout_tasks = held_out(0);
    assert_eq!(
        (0..200)
            .flat_map(|seed| drawn_out_of(&holdout_tasks, seed))
            .collect::<BTreeSet<_>>(),
        holdout_tasks.iter().cloned().collect::<BTreeSet<_>>()
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
fn a_benchmark_with_two_holdout_tasks_is_divided_at_a_half() {
    let holdout_tasks = ["bugs/t01".to_owned(), "bugs/t03".to_owned()];
    assert_eq!(
        holdout::draw_corroboration(&[benchmark("bugs", 3)], &holdout_tasks, 50.0, 7)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn the_holdout_tasks_of_another_benchmark_are_no_part_of_what_is_drawn_from() {
    let holdout_tasks = ["bugs/t01", "bugs/t02", "features/t01", "features/t02"].map(str::to_owned);
    assert_eq!(
        holdout::draw_corroboration(&[benchmark("bugs", 4)], &holdout_tasks, 50.0, 7)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_holdout_task_that_no_benchmark_holds_is_never_drawn() {
    let holdout_tasks = ["bugs/t01", "bugs/t02", "features/t01", "features/t02"].map(str::to_owned);
    assert!((0..50).all(|seed| {
        holdout::draw_corroboration(&[benchmark("bugs", 4)], &holdout_tasks, 50.0, seed).unwrap()[0]
            .starts_with("bugs/")
    }));
}
