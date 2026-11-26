use crate::artificial_selection::focus;

#[test]
fn focus_is_1_at_the_first_diversification() {
    assert_eq!(focus(800, 0), 1.0);
}

#[test]
fn focus_is_2_when_half_the_diversifications_have_started() {
    assert_eq!(focus(800, 400), 2.0);
}

#[test]
fn focus_reaches_the_most_diversifications_at_the_last_diversification() {
    assert_eq!(focus(800, 799), 800.0);
}
