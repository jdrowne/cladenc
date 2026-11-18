use super::ArtificialSelection;
use crate::genealogy::{Genealogy, Harness, HarnessId};

impl ArtificialSelection {
    pub fn best_harness(&self, genealogy: &Genealogy) -> HarnessId {
        genealogy
            .created_harnesses()
            .into_iter()
            .filter(|harness| harness.has_attempted_every(&self.tasks))
            .rev()
            .max_by_key(|harness| harness.holdout_completed())
            .map_or(0, Harness::id)
    }
}
