use cladenc::policy::expansion_is_due;

fn expansions(growth: f64, budget: u64, harnesses: u64) -> Vec<u64> {
    let mut harness_count = harnesses;
    let mut started = 0;
    let mut expansions = Vec::new();
    while started < budget {
        if expansion_is_due(harness_count, started, growth) {
            harness_count += 1;
            expansions.push(started);
        } else {
            started += 1;
        }
    }
    expansions
}

fn harnesses_at_the_end(growth: f64, budget: u64, harnesses: u64) -> u64 {
    harnesses + expansions(growth, budget, harnesses).len() as u64
}

#[test]
fn the_first_job_is_an_evaluation() {
    assert!(!expansion_is_due(1, 0, 0.6));
}

#[test]
fn the_first_expansion_comes_when_one_evaluation_has_started() {
    assert!(expansion_is_due(1, 1, 0.6));
}

#[test]
fn the_expansions_come_when_started_is_1_4_7_11_15_and_20() {
    assert_eq!(expansions(0.6, 21, 1), [1, 4, 7, 11, 15, 20]);
}

#[test]
fn a_budget_of_800_ends_with_56_harnesses() {
    assert_eq!(harnesses_at_the_end(0.6, 800, 1), 56);
}

#[test]
fn a_budget_of_8000_that_starts_from_six_harnesses_ends_with_220() {
    assert_eq!(harnesses_at_the_end(0.6, 8000, 6), 220);
}

#[test]
fn zero_to_the_power_of_zero_counts_as_1() {
    assert!(expansion_is_due(1, 0, 0.0));
}

#[test]
fn at_a_growth_of_0_the_only_expansion_comes_before_any_evaluation() {
    assert_eq!(expansions(0.0, 100, 1), [0]);
}

#[test]
fn at_a_growth_of_1_an_expansion_follows_each_evaluation_but_the_last() {
    assert_eq!(expansions(1.0, 10, 1), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn at_a_growth_of_1_a_budget_of_10_ends_with_10_harnesses() {
    assert_eq!(harnesses_at_the_end(1.0, 10, 1), 10);
}
