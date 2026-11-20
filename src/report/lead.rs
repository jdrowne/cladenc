use serde::Serialize;

use super::leader::Leader;
use crate::genealogy::Harness;

const STEP: i32 = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OnlyLeaderCompleted {
    OnlyBestHarnessCompleted(u64),
    OnlyHarnessAheadSoFarCompleted(u64),
}

impl OnlyLeaderCompleted {
    fn tasks(self) -> u64 {
        match self {
            Self::OnlyBestHarnessCompleted(tasks) | Self::OnlyHarnessAheadSoFarCompleted(tasks) => {
                tasks
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Lead {
    #[serde(flatten)]
    pub(super) only_leader_completed: OnlyLeaderCompleted,
    pub(super) only_primogenitor_completed: u64,
    pub(super) likelihood_by_luck: f64,
}

impl Lead {
    pub(super) fn of(
        leader: Leader,
        ahead: &Harness,
        primogenitor: &Harness,
        corroboration_tasks: &[String],
    ) -> Self {
        let only = |one: &Harness, other: &Harness| {
            corroboration_tasks
                .iter()
                .filter(|task| one.has_completed(task) && !other.has_completed(task))
                .count() as u64
        };
        let only_leader_completed = only(ahead, primogenitor);
        let only_primogenitor_completed = only(primogenitor, ahead);
        Self {
            only_leader_completed: match leader {
                Leader::BestHarness(_) => {
                    OnlyLeaderCompleted::OnlyBestHarnessCompleted(only_leader_completed)
                }
                Leader::HarnessAheadSoFar(_) => {
                    OnlyLeaderCompleted::OnlyHarnessAheadSoFarCompleted(only_leader_completed)
                }
            },
            only_primogenitor_completed,
            likelihood_by_luck: likelihood_by_luck(
                only_leader_completed,
                only_primogenitor_completed,
            ),
        }
    }

    pub(super) fn line(&self, leader: Leader) -> String {
        format!(
            "on the corroboration tasks: only {} completed {}, only the primogenitor completed \
             {}, and luck alone gives a lead at least as large with likelihood {:.3}",
            leader.named(),
            self.only_leader_completed.tasks(),
            self.only_primogenitor_completed,
            self.likelihood_by_luck
        )
    }
}

pub(super) fn likelihood_by_luck(only_leader: u64, only_primogenitor: u64) -> f64 {
    let tosses = only_leader + only_primogenitor;
    if only_leader > only_primogenitor {
        fewer_heads_than(only_primogenitor + 1, tosses)
    } else {
        1.0 - fewer_heads_than(only_leader, tosses)
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
