mod best;
mod wait;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

use crate::genealogy::{Genealogy, Harness, HarnessId, HarnessState};
use wait::Waits;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Procreation { progenitor: HarnessId },
    Attempt { harness: HarnessId, task: String },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ArtificialSelection {
    pub(crate) max_procreations: u64,
    pub(crate) growth: f64,
    pub(crate) tasks: Vec<String>,
    pub(crate) seed: u64,
}

fn focus(max_procreations: u64, started: u64) -> f64 {
    max_procreations as f64 / (max_procreations as f64 - started as f64)
}

fn procreation_is_due(harness_count: u64, started: u64, growth: f64) -> bool {
    harness_count as f64 <= (started as f64).powf(growth)
}

impl ArtificialSelection {
    pub(crate) fn next_action(&self, genealogy: &Genealogy, action: u64) -> Option<Action> {
        self.progenitor(genealogy, action)
            .map(|progenitor| Action::Procreation { progenitor })
            .or_else(|| self.attempt(genealogy))
    }

    fn progenitor(&self, genealogy: &Genealogy, action: u64) -> Option<HarnessId> {
        let procreations = genealogy.procreations_started();
        if procreations >= self.max_procreations || !self.a_procreation_is_due(genealogy) {
            return None;
        }
        let mut generator = ChaCha8Rng::seed_from_u64(self.seed);
        generator.set_stream(action);
        largest_draw(
            &mut generator,
            focus(self.max_procreations, procreations),
            self.harnesses_to_choose_among(genealogy).into_iter(),
        )
        .map(Harness::id)
    }

    fn harnesses_to_choose_among<'a>(&self, genealogy: &'a Genealogy) -> Vec<&'a Harness> {
        let waits = Waits::of(genealogy);
        let (waiting, not_waiting): (Vec<&Harness>, Vec<&Harness>) = genealogy
            .created_harnesses()
            .into_iter()
            .filter(|harness| harness.has_attempted_every(&self.tasks))
            .partition(|harness| waits.has_one_for(harness));
        if not_waiting.is_empty() && self.no_other_work_is_left(genealogy) {
            waits.ending_first(&waiting)
        } else {
            not_waiting
        }
    }

    fn a_procreation_is_due(&self, genealogy: &Genealogy) -> bool {
        procreation_is_due(genealogy.harness_count(), genealogy.started(), self.growth)
            || self.no_other_work_is_left(genealogy)
    }

    fn no_other_work_is_left(&self, genealogy: &Genealogy) -> bool {
        genealogy
            .harnesses()
            .iter()
            .all(|harness| match harness.state() {
                HarnessState::BeingCreated => false,
                HarnessState::Created => harness.has_attempted_every(&self.tasks),
                HarnessState::Failed => true,
            })
    }

    fn attempt(&self, genealogy: &Genealogy) -> Option<Action> {
        genealogy
            .created_harnesses()
            .into_iter()
            .find_map(|harness| {
                harness
                    .tasks_not_given(&self.tasks)
                    .first()
                    .map(|task| Action::Attempt {
                        harness: harness.id(),
                        task: (*task).to_owned(),
                    })
            })
    }
}

fn largest_draw<'a>(
    generator: &mut ChaCha8Rng,
    focus: f64,
    candidates: impl Iterator<Item = &'a Harness>,
) -> Option<&'a Harness> {
    let mut largest: Option<(&Harness, f64)> = None;
    for harness in candidates {
        let draw = Beta::new(
            focus * (1 + harness.clade_completed()) as f64,
            focus * (1 + harness.clade_not_completed()) as f64,
        )
        .expect("both parameters of the Beta distribution are at least 1")
        .sample(generator);
        if largest.is_none_or(|(_, held)| draw > held) {
            largest = Some((harness, draw));
        }
    }
    largest.map(|(harness, _)| harness)
}

#[cfg(test)]
mod tests;
