use statrs::function::beta::inv_beta_reg;

use super::ArtificialSelection;
use crate::genealogy::{Genealogy, HarnessId};

pub fn lower_bound(completed: u64, not_completed: u64, confidence: f64) -> f64 {
    inv_beta_reg(
        (1 + completed) as f64,
        (1 + not_completed) as f64,
        (100.0 - confidence) / 100.0,
    )
}

impl ArtificialSelection {
    pub fn best_harness(&self, genealogy: &Genealogy) -> HarnessId {
        let mut best = (0, f64::NEG_INFINITY);
        for harness in genealogy.created_harnesses() {
            let bound = lower_bound(
                harness.completed(),
                harness.not_completed(),
                self.confidence,
            );
            if bound > best.1 {
                best = (harness.id(), bound);
            }
        }
        best.0
    }
}
