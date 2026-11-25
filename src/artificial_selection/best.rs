use super::ArtificialSelection;
use crate::genealogy::{Genealogy, Harness, HarnessId};

impl ArtificialSelection {
    pub(crate) fn best_harness(&self, genealogy: &Genealogy) -> HarnessId {
        genealogy
            .created_harnesses()
            .into_iter()
            .filter(|harness| !harness.is_a_singleton_control_group_harness())
            .filter(|harness| harness.has_trialled_every(&self.tasks))
            .rev()
            .max_by_key(|harness| harness.selection_completed())
            .map_or(0, Harness::id)
    }
}
