use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::dataset::Dataset;
use crate::holdout::{self, Percentages};
use rand::SeedableRng;
use rand::seq::index;
use rand_chacha::ChaCha8Rng;

const PERCENTAGES: Percentages = Percentages {
    selection: 30.0,
    corroboration: 20.0,
};

fn dataset(name: &str, tasks: usize) -> Dataset {
    Dataset {
        directory: PathBuf::from("/abs").join(name),
        tasks: (1..=tasks).map(|task| format!("t{task:02}")).collect(),
    }
}

fn datasets() -> [Dataset; 2] {
    [dataset("bugs", 10), dataset("features", 4)]
}

fn drawn(seed: u64) -> Vec<String> {
    holdout::draw(&datasets(), PERCENTAGES, seed)
        .unwrap()
        .holdout_tasks
}

fn drawn_of(name: &str, seed: u64) -> Vec<String> {
    drawn(seed)
        .into_iter()
        .filter(|task| task.starts_with(name))
        .collect()
}

fn every_task() -> BTreeSet<String> {
    datasets()
        .iter()
        .flat_map(|dataset| dataset.tasks.iter().map(|id| dataset.task(id)))
        .collect()
}

#[test]
fn the_holdout_tasks_of_each_dataset_are_as_many_as_its_selection_and_corroboration_tasks() {
    assert_eq!(
        (drawn_of("bugs/", 7).len(), drawn_of("features/", 7).len()),
        (5, 2)
    );
}

#[test]
fn a_holdout_task_is_a_task_of_a_dataset_named_by_its_dataset_and_its_id() {
    assert!(drawn(7).iter().all(|task| every_task().contains(task)));
}

#[test]
fn no_task_is_drawn_twice() {
    assert_eq!(drawn(7).into_iter().collect::<BTreeSet<_>>().len(), 7);
}

#[test]
fn the_holdout_tasks_are_in_the_order_of_the_datasets_and_of_their_tasks() {
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
fn every_task_is_held_out_under_one_seed_or_another() {
    assert_eq!(
        (0..200).flat_map(drawn).collect::<BTreeSet<_>>(),
        every_task()
    );
}

#[test]
fn the_tasks_are_drawn_from_the_last_stream_of_the_seeded_generator() {
    let mut generator = ChaCha8Rng::seed_from_u64(7);
    generator.set_stream(u64::MAX);
    let mut places = index::sample(&mut generator, 10, 5).into_vec();
    places.sort_unstable();
    assert_eq!(
        drawn_of("bugs/", 7),
        places
            .iter()
            .map(|place| format!("bugs/t{:02}", place + 1))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_dataset_of_three_tasks_can_have_two_holdout_tasks() {
    assert_eq!(
        holdout::draw(
            &[dataset("bugs", 3)],
            Percentages {
                selection: 30.0,
                corroboration: 30.0
            },
            7
        )
        .unwrap()
        .holdout_tasks
        .len(),
        2
    );
}
