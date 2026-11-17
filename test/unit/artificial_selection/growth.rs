use cladenc::artificial_selection::procreation_is_due;

fn procreations(growth: f64, attempts: u64, harnesses: u64) -> Vec<u64> {
    let mut harness_count = harnesses;
    let mut started = 0;
    let mut procreations = Vec::new();
    while started < attempts {
        if procreation_is_due(harness_count, started, growth) {
            harness_count += 1;
            procreations.push(started);
        } else {
            started += 1;
        }
    }
    procreations
}

fn harnesses_at_the_end(growth: f64, attempts: u64, harnesses: u64) -> u64 {
    harnesses + procreations(growth, attempts, harnesses).len() as u64
}

#[test]
fn the_first_action_is_an_attempt() {
    assert!(!procreation_is_due(1, 0, 0.6));
}

#[test]
fn the_first_procreation_comes_when_one_attempt_has_started() {
    assert!(procreation_is_due(1, 1, 0.6));
}

#[test]
fn the_procreations_come_when_started_is_1_4_7_11_15_and_20() {
    assert_eq!(procreations(0.6, 21, 1), [1, 4, 7, 11, 15, 20]);
}

#[test]
fn the_growth_lets_800_attempts_end_with_56_harnesses() {
    assert_eq!(harnesses_at_the_end(0.6, 800, 1), 56);
}

#[test]
fn the_growth_lets_8000_attempts_that_start_from_six_harnesses_end_with_220() {
    assert_eq!(harnesses_at_the_end(0.6, 8000, 6), 220);
}

#[test]
fn zero_to_the_power_of_zero_counts_as_1() {
    assert!(procreation_is_due(1, 0, 0.0));
}

#[test]
fn at_a_growth_of_0_the_only_procreation_the_growth_lets_start_comes_before_any_attempt() {
    assert_eq!(procreations(0.0, 100, 1), [0]);
}

#[test]
fn at_a_growth_of_1_a_procreation_follows_each_attempt_but_the_last() {
    assert_eq!(procreations(1.0, 10, 1), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn at_a_growth_of_1_the_growth_lets_10_attempts_end_with_10_harnesses() {
    assert_eq!(harnesses_at_the_end(1.0, 10, 1), 10);
}
