use std::collections::BTreeMap;

use crate::genealogy::{Genealogy, Harness, HarnessId, HarnessState};

#[derive(Clone, Copy, Default)]
struct CreatedNoChild {
    in_a_row: u32,
    last: HarnessId,
}

impl CreatedNoChild {
    fn followed_by(self, child: &Harness) -> Self {
        match child.state() {
            HarnessState::BeingCreated => self,
            HarnessState::Created => Self::default(),
            HarnessState::Failed => Self {
                in_a_row: self.in_a_row + 1,
                last: child.id(),
            },
        }
    }

    fn last_procreation_waited_out(self) -> u64 {
        self.in_a_row.checked_sub(1).map_or(0, |doublings| {
            self.last
                .saturating_add(1_u64.checked_shl(doublings).unwrap_or(u64::MAX))
        })
    }
}

pub(super) struct Waits {
    next_procreation: u64,
    last_procreation_waited_out: BTreeMap<HarnessId, u64>,
}

impl Waits {
    pub(super) fn of(genealogy: &Genealogy) -> Self {
        let mut created_no_child: BTreeMap<HarnessId, CreatedNoChild> = BTreeMap::new();
        for child in genealogy.harnesses() {
            if let Some(progenitor) = child.progenitor() {
                let so_far = created_no_child.entry(progenitor).or_default();
                *so_far = so_far.followed_by(child);
            }
        }
        Self {
            next_procreation: genealogy.harness_count(),
            last_procreation_waited_out: created_no_child
                .into_iter()
                .map(|(progenitor, count)| (progenitor, count.last_procreation_waited_out()))
                .collect(),
        }
    }

    pub(super) fn has_one_for(&self, harness: &Harness) -> bool {
        self.last_procreation_waited_out_by(harness) >= self.next_procreation
    }

    pub(super) fn ending_first<'a>(&self, harnesses: &[&'a Harness]) -> Vec<&'a Harness> {
        let first = harnesses
            .iter()
            .map(|harness| self.last_procreation_waited_out_by(harness))
            .min();
        harnesses
            .iter()
            .copied()
            .filter(|harness| Some(self.last_procreation_waited_out_by(harness)) == first)
            .collect()
    }

    fn last_procreation_waited_out_by(&self, harness: &Harness) -> u64 {
        self.last_procreation_waited_out
            .get(&harness.id())
            .copied()
            .unwrap_or(0)
    }
}
