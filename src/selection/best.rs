use statrs::function::beta::inv_beta_reg;

use super::Selection;
use crate::genealogy::{Genealogy, HarnessId};

pub fn lower_bound(successes: u64, failures: u64, confidence: f64) -> f64 {
    inv_beta_reg(
        (1 + successes) as f64,
        (1 + failures) as f64,
        (100.0 - confidence) / 100.0,
    )
}

impl Selection {
    pub fn best_harness(&self, genealogy: &Genealogy) -> HarnessId {
        let mut best = (0, f64::NEG_INFINITY);
        for harness in genealogy.created_harnesses() {
            let bound = lower_bound(harness.successes(), harness.failures(), self.confidence);
            if bound > best.1 {
                best = (harness.id(), bound);
            }
        }
        best.0
    }
}
