mod best;

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

use crate::genealogy::{Genealogy, Harness, HarnessId};

pub use best::lower_bound;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Procreation { progenitor: HarnessId },
    Assessment { harness: HarnessId, task: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtificialSelection {
    pub budget: u64,
    pub growth: f64,
    pub confidence: f64,
    pub tasks: Vec<String>,
    pub seed: u64,
}

pub fn focus(budget: u64, started: u64) -> f64 {
    budget as f64 / (budget as f64 - started as f64)
}

pub fn procreation_is_due(harness_count: u64, started: u64, growth: f64) -> bool {
    harness_count as f64 <= (started as f64).powf(growth)
}

impl ArtificialSelection {
    pub fn next_action(&self, genealogy: &Genealogy, action: u64) -> Option<Action> {
        let started = genealogy.started();
        if started >= self.budget {
            return None;
        }
        let focus = focus(self.budget, started);
        let mut generator = ChaCha8Rng::seed_from_u64(self.seed);
        generator.set_stream(action);
        let created = genealogy.created_harnesses();
        if procreation_is_due(genealogy.harness_count(), started, self.growth) {
            let clades = created.iter().map(|harness| {
                (
                    *harness,
                    harness.clade_successes(),
                    harness.clade_failures(),
                )
            });
            return largest_draw(&mut generator, focus, clades).map(|progenitor| {
                Action::Procreation {
                    progenitor: progenitor.id(),
                }
            });
        }
        let with_a_task_left = created
            .iter()
            .filter(|harness| !harness.tasks_not_given(&self.tasks).is_empty())
            .map(|harness| (*harness, harness.successes(), harness.failures()));
        let assessed = largest_draw(&mut generator, focus, with_a_task_left)?;
        let left = assessed.tasks_not_given(&self.tasks);
        let task = left[generator.random_range(0..left.len())].to_owned();
        Some(Action::Assessment {
            harness: assessed.id(),
            task,
        })
    }
}

fn largest_draw<'a>(
    generator: &mut ChaCha8Rng,
    focus: f64,
    candidates: impl Iterator<Item = (&'a Harness, u64, u64)>,
) -> Option<&'a Harness> {
    let mut largest: Option<(&Harness, f64)> = None;
    for (harness, successes, failures) in candidates {
        let draw = Beta::new(
            focus * (1 + successes) as f64,
            focus * (1 + failures) as f64,
        )
        .expect("both parameters of the Beta distribution are at least 1")
        .sample(generator);
        if largest.is_none_or(|(_, held)| draw > held) {
            largest = Some((harness, draw));
        }
    }
    largest.map(|(harness, _)| harness)
}
