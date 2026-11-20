use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use super::lead::likelihood_by_luck;
use crate::genealogy::{Genealogy, Harness, HarnessState};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Experiments {
    pub(super) conducted: u64,
    pub(super) progenitors_child_created: u64,
    pub(super) singleton_control_group_harness_created: u64,
    pub(super) both_created: u64,
    pub(super) only_progenitors_child_completed: u64,
    pub(super) only_singleton_control_group_harness_completed: u64,
    pub(super) likelihood_by_luck: Option<f64>,
}

impl Experiments {
    pub(super) fn of(genealogy: &Genealogy, corroboration_tasks: &[String]) -> Self {
        let harnesses = genealogy.harnesses();
        let experiments: Vec<(&Harness, &Harness)> = harnesses
            .iter()
            .filter_map(|singleton_control_group_harness| {
                singleton_control_group_harness
                    .experiment()
                    .and_then(|experiment| harnesses.get(experiment.child as usize))
                    .map(|child| (child, singleton_control_group_harness))
            })
            .collect();
        let both_created: Vec<(&Harness, &Harness)> = experiments
            .iter()
            .copied()
            .filter(|(child, singleton_control_group_harness)| {
                created(child) && created(singleton_control_group_harness)
            })
            .collect();
        let only_progenitors_child_completed: u64 = both_created
            .iter()
            .map(|(child, singleton_control_group_harness)| {
                completed_by_one_alone(child, singleton_control_group_harness, corroboration_tasks)
            })
            .sum();
        let only_singleton_control_group_harness_completed: u64 = both_created
            .iter()
            .map(|(child, singleton_control_group_harness)| {
                completed_by_one_alone(singleton_control_group_harness, child, corroboration_tasks)
            })
            .sum();
        Self {
            conducted: experiments.len() as u64,
            progenitors_child_created: experiments
                .iter()
                .filter(|(child, _)| created(child))
                .count() as u64,
            singleton_control_group_harness_created: experiments
                .iter()
                .filter(|(_, singleton_control_group_harness)| {
                    created(singleton_control_group_harness)
                })
                .count() as u64,
            both_created: both_created.len() as u64,
            only_progenitors_child_completed,
            only_singleton_control_group_harness_completed,
            likelihood_by_luck: (!both_created.is_empty()).then(|| {
                likelihood_by_luck(
                    only_progenitors_child_completed,
                    only_singleton_control_group_harness_completed,
                )
            }),
        }
    }
}

impl Display for Experiments {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let told_apart = self.likelihood_by_luck.map_or_else(
            || {
                ": no experiment created both the progenitor's child and the singleton control \
                 group harness, so there are no two harnesses to compare"
                    .to_owned()
            },
            |likelihood| {
                format!(
                    " where both were created: only the progenitor's child completed {}, only \
                     the singleton control group harness completed {}, and luck alone gives a \
                     lead at least as large with likelihood {likelihood:.3}",
                    self.only_progenitors_child_completed,
                    self.only_singleton_control_group_harness_completed
                )
            },
        );
        write!(
            formatter,
            "experiments: conducted {}, the progenitor's child created in {}, the singleton \
             control group harness created in {}, both created in {}\non the corroboration tasks \
             of the experiments{told_apart}",
            self.conducted,
            self.progenitors_child_created,
            self.singleton_control_group_harness_created,
            self.both_created
        )
    }
}

fn created(harness: &Harness) -> bool {
    harness.state() == HarnessState::Created
}

fn completed_by_one_alone(one: &Harness, other: &Harness, tasks: &[String]) -> u64 {
    tasks
        .iter()
        .filter(|task| {
            one.has_attempted(task)
                && other.has_attempted(task)
                && one.has_completed(task)
                && !other.has_completed(task)
        })
        .count() as u64
}
