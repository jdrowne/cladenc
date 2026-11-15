use cladenc::policy::lower_bound;

#[test]
fn a_harness_with_no_evaluations_has_a_lower_bound_of_one_hundredth() {
    assert!((lower_bound(0, 0, 99.0) - 0.01).abs() < 1e-9);
}

#[test]
fn a_harness_with_a_single_success_has_a_lower_bound_of_one_tenth() {
    assert!((lower_bound(1, 0, 99.0) - 0.1).abs() < 1e-9);
}

#[test]
fn a_harness_with_a_single_failure_has_a_lower_bound_of_1_less_the_root_of_99_hundredths() {
    assert!((lower_bound(0, 1, 99.0) - (1.0 - 0.99_f64.sqrt())).abs() < 1e-9);
}

#[test]
fn more_successes_raise_the_lower_bound() {
    assert!(lower_bound(10, 0, 99.0) > lower_bound(1, 0, 99.0));
}

#[test]
fn the_same_rate_over_more_evaluations_has_a_higher_lower_bound() {
    assert!(lower_bound(30, 10, 99.0) > lower_bound(3, 1, 99.0));
}

#[test]
fn a_larger_confidence_lowers_the_lower_bound() {
    assert!(lower_bound(3, 1, 99.0) < lower_bound(3, 1, 90.0));
}
