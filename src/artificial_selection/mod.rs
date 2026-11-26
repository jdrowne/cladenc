mod best;
mod wait;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};

use crate::genealogy::{Genealogy, Harness, HarnessId, HarnessState};
use wait::Waits;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Diversification { progenitor: HarnessId },
    Trial { harness: HarnessId, task: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtificialSelection {
    pub(crate) max_diversifications: u64,
    pub(crate) tasks: Vec<String>,
    pub(crate) seed: u64,
}

fn focus(max_diversifications: u64, started: u64) -> f64 {
    max_diversifications as f64 / (max_diversifications as f64 - started as f64)
}

impl ArtificialSelection {
    pub(crate) fn next_action(&self, genealogy: &Genealogy, action: u64) -> Option<Action> {
        self.progenitor(genealogy, action)
            .map(|progenitor| Action::Diversification { progenitor })
            .or_else(|| self.trial(genealogy))
    }

    fn progenitor(&self, genealogy: &Genealogy, action: u64) -> Option<HarnessId> {
        let diversifications = genealogy.diversifications_started();
        if diversifications >= self.max_diversifications || !self.no_other_work_is_left(genealogy) {
            return None;
        }
        let mut generator = ChaCha8Rng::seed_from_u64(self.seed);
        generator.set_stream(action);
        largest_draw(
            &mut generator,
            focus(self.max_diversifications, diversifications),
            harnesses_to_choose_among(genealogy).into_iter(),
        )
        .map(Harness::id)
    }

    fn no_other_work_is_left(&self, genealogy: &Genealogy) -> bool {
        genealogy
            .harnesses()
            .iter()
            .filter(|harness| !harness.is_a_singleton_control_group_harness())
            .all(|harness| match harness.state() {
                HarnessState::BeingCreated => false,
                HarnessState::Created => harness.has_trialled_every(&self.tasks),
                HarnessState::Unchanged | HarnessState::Failed => true,
            })
    }

    fn trial(&self, genealogy: &Genealogy) -> Option<Action> {
        genealogy
            .created_harnesses()
            .into_iter()
            .find_map(|harness| {
                harness
                    .tasks_not_given(&self.tasks)
                    .first()
                    .map(|task| Action::Trial {
                        harness: harness.id(),
                        task: (*task).to_owned(),
                    })
            })
    }
}

fn harnesses_to_choose_among(genealogy: &Genealogy) -> Vec<&Harness> {
    let waits = Waits::of(genealogy);
    let (waiting, not_waiting): (Vec<&Harness>, Vec<&Harness>) = genealogy
        .created_harnesses()
        .into_iter()
        .filter(|harness| !harness.is_a_singleton_control_group_harness())
        .partition(|harness| waits.has_one_for(harness));
    if not_waiting.is_empty() {
        waits.ending_first(&waiting)
    } else {
        not_waiting
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
