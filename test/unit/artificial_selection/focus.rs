use cladenc::artificial_selection::focus;

#[test]
fn focus_is_1_when_no_assessment_has_started() {
    assert_eq!(focus(800, 0), 1.0);
}

#[test]
fn focus_is_2_when_half_the_budget_has_started() {
    assert_eq!(focus(800, 400), 2.0);
}

#[test]
fn focus_reaches_the_budget_at_the_last_assessment() {
    assert_eq!(focus(800, 799), 800.0);
}
