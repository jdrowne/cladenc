use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::genealogy::Harness;

const STEP: i32 = 512;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Lead {
    pub(super) only_best_harness_completed: u64,
    pub(super) only_primogenitor_completed: u64,
    pub(super) likelihood_by_luck: f64,
}

impl Lead {
    pub(super) fn of(
        best_harness: &Harness,
        primogenitor: &Harness,
        corroboration_tasks: &[String],
    ) -> Self {
        let only = |one: &Harness, other: &Harness| {
            corroboration_tasks
                .iter()
                .filter(|task| one.has_completed(task) && !other.has_completed(task))
                .count() as u64
        };
        let only_best_harness_completed = only(best_harness, primogenitor);
        let only_primogenitor_completed = only(primogenitor, best_harness);
        Self {
            only_best_harness_completed,
            only_primogenitor_completed,
            likelihood_by_luck: likelihood_by_luck(
                only_best_harness_completed,
                only_primogenitor_completed,
            ),
        }
    }
}

impl Display for Lead {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "on the corroboration tasks: only the best harness completed {}, only the primogenitor \
             completed {}, and luck alone gives a lead at least as large with likelihood {:.3}",
            self.only_best_harness_completed,
            self.only_primogenitor_completed,
            self.likelihood_by_luck
        )
    }
}

pub(super) fn likelihood_by_luck(only_best_harness: u64, only_primogenitor: u64) -> f64 {
    let tosses = only_best_harness + only_primogenitor;
    if only_best_harness > only_primogenitor {
        fewer_heads_than(only_primogenitor + 1, tosses)
    } else {
        1.0 - fewer_heads_than(only_best_harness, tosses)
    }
}

fn fewer_heads_than(heads: u64, tosses: u64) -> f64 {
    let step = 2.0_f64.powi(STEP);
    let tosses_in_a_step = u64::from(STEP.unsigned_abs());
    let mut ways = 1.0_f64;
    let mut sum = 0.0_f64;
    let mut halvings = tosses;
    for fewer in 0..heads {
        sum += ways;
        ways = ways * (tosses - fewer) as f64 / (fewer + 1) as f64;
        if ways > step {
            ways /= step;
            sum /= step;
            halvings = halvings.saturating_sub(tosses_in_a_step);
        }
    }
    while halvings >= tosses_in_a_step {
        sum /= step;
        halvings -= tosses_in_a_step;
    }
    sum / 2.0_f64.powi(i32::try_from(halvings).unwrap_or(STEP))
}
